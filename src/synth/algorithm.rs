use super::operator::Operator; // Assuming Operator is defined in a parent module
use std::collections::HashMap;

// --- Internal Graph Structures (Used by Algorithm::process) ---

/// Represents an Operator at a specific 'unrolled' feedback level in the DAG.
#[derive(Debug)]
struct UnrolledNode {
    original_op_index: usize, // Index into the original operators array
    // Indices of required input nodes within the `AlgorithmProcessor::nodes` vector.
    input_node_indices: Vec<usize>,
}

/// Holds the pre-built DAG and operator references for processing.
struct AlgorithmProcessor<'a> {
    nodes: Vec<UnrolledNode>,
    operators: &'a [Operator],
    // Indices in `self.nodes` corresponding to the final output of carrier operators.
    carrier_node_indices: Vec<usize>,
}

// --- Public Algorithm Struct (Matches Original API) ---

/// Defines the operator connections and processing logic for FM synthesis.
#[derive(Clone, Debug)]
pub struct Algorithm {
    /// Adjacency matrix: `matrix[i][j] = Some(N)` means op `j` modulates op `i`.
    pub matrix: Vec<Vec<Option<usize>>>,
    pub carriers: Vec<usize>,
}

// --- Implementation ---

impl Algorithm {
    /// Creates a new algorithm definition.
    pub fn new(matrix: Vec<Vec<Option<usize>>>, carriers: Vec<usize>) -> Result<Self, String> {
        let num_ops = matrix.len();
        if num_ops > 0 && !matrix.iter().all(|row| row.len() == num_ops) {
            return Err("Adjacency matrix must be square.".to_string());
        }
        if let Some(max_carrier) = carriers.iter().max() {
            if num_ops == 0 || *max_carrier >= num_ops {
                // Also check num_ops > 0 for max_carrier check
                return Err(format!(
                    "Carrier index {} out of bounds for {} operators.",
                    max_carrier, num_ops
                ));
            }
        }
        // Basic validation passed. More could be added (e.g., check matrix content indices).
        Ok(Self { matrix, carriers })
    }

    /// Default: Single carrier (operator 0), no modulation.
    pub fn default_simple(num_operators: usize) -> Result<Self, String> {
        let matrix = vec![vec![None; num_operators]; num_operators];
        let carriers = if num_operators > 0 { vec![0] } else { vec![] };
        Self::new(matrix, carriers)
    }

    /// Default: 2-Operator stack (1 -> 0).
    pub fn default_stack_2(num_operators: usize) -> Result<Self, String> {
        if num_operators < 2 {
            return Self::default_simple(num_operators);
        }
        let mut matrix = vec![vec![None; num_operators]; num_operators];
        matrix[0][1] = Some(1); // Standard connection
        Self::new(matrix, vec![0])
    }

    /// Default: Operator 0 self-feedback (1 pass).
    pub fn default_feedback_1(num_operators: usize) -> Result<Self, String> {
        if num_operators < 1 {
            return Self::default_simple(num_operators);
        }
        let mut matrix = vec![vec![None; num_operators]; num_operators];
        matrix[0][0] = Some(2); // N=2 => 1 feedback level
        Self::new(matrix, vec![0])
    }

    /// Processes the algorithm, filling the output buffer.
    /// Builds an unrolled DAG internally and processes it recursively.
    pub fn process(
        &self,
        operators: &[Operator],
        base_frequency: f32,
        output: &mut [f32],
        sample_rate: f32,
        start_sample_index: u64,
    ) {
        let buffer_size = output.len();
        output.fill(0.0); // Clear output initially

        let num_operators = operators.len();
        if buffer_size == 0 || num_operators == 0 || self.matrix.len() != num_operators {
            if self.matrix.len() != num_operators && num_operators > 0 {
                eprintln!("Warning: Algorithm matrix size ({}) differs from number of operators ({}). No processing.", self.matrix.len(), num_operators);
            }
            return;
        }

        // 1. Build the internal unrolled graph representation.
        match Self::build_processor(&self.matrix, &self.carriers, operators) {
            Ok(processor) => {
                // 2. Process the built graph.
                let mut modulation_input_buffer: Vec<f32> = vec![0.0; buffer_size];

                for &carrier_node_idx in &processor.carrier_node_indices {
                    match processor.process_node_recursive(
                        carrier_node_idx,
                        base_frequency,
                        sample_rate,
                        start_sample_index,
                        buffer_size,
                        &mut modulation_input_buffer,
                    ) {
                        Ok(carrier_output) => {
                            for (out_sample, carrier_sample) in
                                output.iter_mut().zip(carrier_output.iter())
                            {
                                *out_sample += *carrier_sample;
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "Error processing graph node {}: {}. Output might be incomplete.",
                                carrier_node_idx, e
                            );
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to build internal processing graph: {}", e);
            }
        }
    }

    // --- Internal Graph Building Logic (Moved from UnrolledAlgorithmGraph) ---

    /// Builds the internal processor containing the unrolled DAG.
    fn build_processor<'a>(
        matrix: &[Vec<Option<usize>>],
        carriers: &[usize],
        operators: &'a [Operator],
    ) -> Result<AlgorithmProcessor<'a>, String> {
        let num_ops = operators.len(); // Already validated in process entry

        let mut final_nodes: Vec<UnrolledNode> = Vec::new();
        let mut created_nodes_map: HashMap<(usize, usize), usize> = HashMap::new();

        let max_level = matrix
            .iter()
            .flatten()
            .filter_map(|&opt_n| opt_n)
            .map(|n| n.saturating_sub(1))
            .max()
            .unwrap_or(0);

        let mut final_carrier_indices = Vec::with_capacity(carriers.len());
        for &op_idx in carriers {
            let carrier_node_idx = Self::get_or_create_node_index(
                op_idx,
                max_level,
                matrix,
                &mut final_nodes,
                &mut created_nodes_map,
            )?;
            final_carrier_indices.push(carrier_node_idx);
        }

        Ok(AlgorithmProcessor {
            nodes: final_nodes,
            operators,
            carrier_node_indices: final_carrier_indices,
        })
    }

    /// Recursive helper to build/get node indices for the DAG.
    fn get_or_create_node_index(
        target_op_idx: usize,
        target_level: usize,
        matrix: &[Vec<Option<usize>>],
        final_nodes: &mut Vec<UnrolledNode>,
        created_nodes_map: &mut HashMap<(usize, usize), usize>,
    ) -> Result<usize, String> {
        let node_key = (target_op_idx, target_level);
        if let Some(&idx) = created_nodes_map.get(&node_key) {
            return Ok(idx);
        }

        let current_node_idx = final_nodes.len();
        final_nodes.push(UnrolledNode {
            original_op_index: target_op_idx,
            input_node_indices: Vec::new(),
        });
        created_nodes_map.insert(node_key, current_node_idx);

        let mut input_indices_for_current = Vec::new();
        let num_ops = matrix.len();

        for source_op_idx in 0..num_ops {
            if let Some(connection_n) = matrix[target_op_idx][source_op_idx] {
                if connection_n == 0 {
                    continue;
                }

                let source_level_required = if connection_n == 1 {
                    0
                } else if target_level > 0 {
                    target_level - 1
                } else {
                    continue;
                };

                let input_node_idx = Self::get_or_create_node_index(
                    source_op_idx,
                    source_level_required,
                    matrix,
                    final_nodes,
                    created_nodes_map,
                )?;
                input_indices_for_current.push(input_node_idx);
            }
        }
        final_nodes[current_node_idx].input_node_indices = input_indices_for_current;
        Ok(current_node_idx)
    }
}

impl<'a> AlgorithmProcessor<'a> {
    /// Recursively processes a single node in the pre-built unrolled DAG.
    #[allow(clippy::too_many_arguments)]
    fn process_node_recursive(
        &self,
        node_idx: usize, // Index in self.nodes
        base_frequency: f32,
        sample_rate: f32,
        start_sample_index: u64,
        buffer_size: usize,
        modulation_input: &mut Vec<f32>,
    ) -> Result<Vec<f32>, String> {
        if node_idx >= self.nodes.len() {
            return Err(format!("Invalid node index {}.", node_idx));
        }
        let node = &self.nodes[node_idx];

        modulation_input.fill(0.0);

        for &input_node_idx in &node.input_node_indices {
            match self.process_node_recursive(
                input_node_idx,
                base_frequency,
                sample_rate,
                start_sample_index,
                buffer_size,
                modulation_input,
            ) {
                Ok(mod_output) => {
                    if input_node_idx < self.nodes.len() {
                        let modulator_op_idx = self.nodes[input_node_idx].original_op_index;
                        if modulator_op_idx < self.operators.len() {
                            let mod_strength = self.operators[modulator_op_idx].modulation_index;
                            for i in 0..buffer_size {
                                modulation_input[i] += mod_output[i] * mod_strength;
                            }
                        } else {
                            return Err(format!(
                                "Invalid original operator index {} in node {}.",
                                modulator_op_idx, input_node_idx
                            ));
                        }
                    } else {
                        return Err(format!(
                            "Invalid input node index {} for node {}.",
                            input_node_idx, node_idx
                        ));
                    }
                }
                Err(e) => {
                    return Err(format!(
                        "Error processing input node {} for node {}: {}",
                        input_node_idx, node_idx, e
                    ));
                }
            }
        }

        let current_op_idx = node.original_op_index;
        if current_op_idx >= self.operators.len() {
            return Err(format!(
                "Invalid original operator index {} in node {}.",
                current_op_idx, node_idx
            ));
        }

        let mut current_op_output = vec![0.0; buffer_size];
        self.operators[current_op_idx].process(
            base_frequency,
            &mut current_op_output,
            modulation_input,
            sample_rate,
            start_sample_index,
        );

        Ok(current_op_output)
    }
}

