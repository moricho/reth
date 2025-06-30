//! The implementation of parallel sparse MPT.
//!
//! # Parallel Sparse Trie Structure
//!
//! The [`ParallelSparseTrie`] implements a two-level trie structure that enables parallel
//! processing of trie updates. This structure is designed to optimize performance by
//! allowing independent subtries to be updated concurrently.
//!
//! ## Structure Overview
//!
//! The trie is split into two levels:
//!
//! 1. **Upper Subtrie**: Contains nodes with paths of length less than [`UPPER_TRIE_MAX_DEPTH`] (2
//!    nibbles)
//! 2. **Lower Subtries**: 256 separate subtries for paths of length 2 or more nibbles
//!
//! ### Upper Subtrie
//!
//! The upper subtrie contains at most 17 nodes:
//! - 1 root node (empty path)
//! - Up to 16 child nodes (paths of length 1: `0x0` through `0xf`)
//!
//! Paths in the upper subtrie:
//! - `0x` (root)
//! - `0x0`, `0x1`, `0x2`, ..., `0xf` (children)
//!
//! ### Lower Subtries
//!
//! There are exactly 256 lower subtries, indexed from 0 to 255. Each lower subtrie handles
//! paths that start with a specific 2-nibble prefix.
//!
//! The index of a lower subtrie is calculated from the first 2 nibbles of the path:
//! - Paths starting with `0x00` go to subtrie 0
//! - Paths starting with `0x01` go to subtrie 1
//! - ...
//! - Paths starting with `0xff` go to subtrie 255
//!
//! For example:
//! - Path `0x1234...` goes to lower subtrie 18 (0x12 = 18 in decimal)
//! - Path `0xabcd...` goes to lower subtrie 171 (0xab = 171 in decimal)
//!
//! ## Node Distribution
//!
//! ### Path Length Rules
//!
//! - **Upper subtrie**: Paths with length < 2 nibbles
//! - **Lower subtries**: Paths with length ≥ 2 nibbles
//!
//! ### Examples
//!
//! ```
//! // Upper subtrie nodes
//! 0x        -> root node
//! 0x1       -> child node
//! 0xa       -> child node
//!
//! // Lower subtrie nodes (examples)
//! 0x12      -> lower subtrie 18
//! 0x123     -> lower subtrie 18
//! 0x1234    -> lower subtrie 18
//! 0xab      -> lower subtrie 171
//! 0xabcd    -> lower subtrie 171
//! ```
//!
//! ## Operations
//!
//! ### Adding Leaves
//!
//! When adding a leaf node:
//! 1. Determine if the path belongs to upper or lower subtrie based on path length
//! 2. If upper subtrie: add directly to `upper_subtrie`
//! 3. If lower subtrie:
//!    - Calculate subtrie index from first 2 nibbles
//!    - Create lower subtrie if it doesn't exist
//!    - Add node to the appropriate lower subtrie
//!
//! ### Removing Nodes
//!
//! When removing a node:
//! 1. Locate the node in the appropriate subtrie (upper or lower)
//! 2. Remove the node and update the trie structure
//! 3. If a lower subtrie becomes empty, it can be removed
//! 4. Update parent nodes as needed
//!
//! ### Updating Hashes
//!
//! Hash updates are performed in two phases:
//!
//! 1. **Lower subtrie hash updates** (`update_lower_subtrie_hashes`):
//!    - Process all changed lower subtries in parallel
//!    - Calculate hashes for all nodes within each subtrie
//!    - Update the root hash of each lower subtrie
//!
//! 2. **Upper subtrie hash updates** (`update_upper_subtrie_hashes`):
//!    - Process the upper subtrie using both upper and lower subtrie nodes
//!    - Calculate the final root hash
//!
//! ## Performance Benefits
//!
//! This structure provides several performance advantages:
//!
//! 1. **Parallel Processing**: Lower subtries can be updated independently and in parallel
//! 2. **Reduced Contention**: Upper subtrie has limited size (max 17 nodes), reducing lock
//!    contention
//! 3. **Efficient Updates**: Only changed subtries need to be processed during hash updates
//! 4. **Scalability**: The structure scales well with large numbers of leaf nodes
//!
//! ## Implementation Details
//!
//! - The [`SparseSubtrieType`] enum determines which subtrie a path belongs to
//! - The `path_subtrie_index_unchecked` function calculates the lower subtrie index
//! - Each subtrie maintains its own node collection and update tracking
//! - The `prefix_set` tracks which parts of the trie have been modified

#![cfg_attr(not(test), warn(unused_crate_dependencies))]

mod trie;
pub use trie::*;
