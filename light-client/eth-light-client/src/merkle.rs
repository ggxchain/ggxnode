use blake2::{Blake2s256, Digest};
use ethers::types::H256;
use eyre::{Report, Result};
use merkle_cbt::merkle_tree::{Merge, CBMT};

struct MergeH256 {}

impl Merge for MergeH256 {
    type Item = H256;

    fn merge(left: &Self::Item, right: &Self::Item) -> Self::Item {
        let left_and_right = left
            .as_bytes()
            .iter()
            .chain(right.as_bytes().iter())
            .cloned()
            .collect::<Vec<_>>();

        H256::from_slice(Blake2s256::digest(left_and_right).as_slice())
    }
}

#[allow(non_camel_case_types)]
type CBMT_H256 = CBMT<H256, MergeH256>;

pub fn root(hashes: &[H256]) -> H256 {
    CBMT_H256::build_merkle_root(hashes)
}

pub fn verify(hashes: &[H256], indices: &[u32], proof_leaves: &[H256]) -> Result<bool> {
    let root = CBMT_H256::build_merkle_root(hashes);
    let proof = CBMT_H256::build_merkle_proof(hashes, indices)
        .ok_or(Report::msg("Could not build proof"))?;
    Ok(proof.verify(&root, proof_leaves))
}

#[cfg(test)]
mod tests {

    use super::*;

    use ethers::abi::AbiDecode;
    use hex_literal::hex;

    #[test]
    fn root_of_emptry_tree_test() {
        assert_eq!(
            root(&[]),
            H256::decode(hex!(
                "0000000000000000000000000000000000000000000000000000000000000000"
            ))
            .unwrap()
        )
    }

    #[test]
    fn root_of_not_emptry_tree_test() {
        let hashes = [
            H256::decode(hex!(
                "0000000000000000000000000000000000000000000000000000000000000001"
            ))
            .unwrap(),
            H256::decode(hex!(
                "0000000000000000000000000000000000000000000000000000000000000002"
            ))
            .unwrap(),
            H256::decode(hex!(
                "0000000000000000000000000000000000000000000000000000000000000003"
            ))
            .unwrap(),
        ];
        assert_eq!(
            root(&hashes),
            H256::decode(hex!(
                "c589709931c1a867f903dec1c25821e3893ce05c621fbc51fb568efafde841ab"
            ))
            .unwrap()
        )
    }

    #[test]
    fn verify_test() {
        let hashes = [
            H256::decode(hex!(
                "0000000000000000000000000000000000000000000000000000000000000001"
            ))
            .unwrap(),
            H256::decode(hex!(
                "0000000000000000000000000000000000000000000000000000000000000002"
            ))
            .unwrap(),
            H256::decode(hex!(
                "0000000000000000000000000000000000000000000000000000000000000003"
            ))
            .unwrap(),
            H256::decode(hex!(
                "0000000000000000000000000000000000000000000000000000000000000004"
            ))
            .unwrap(),
            H256::decode(hex!(
                "0000000000000000000000000000000000000000000000000000000000000005"
            ))
            .unwrap(),
        ];
        let indices = [3u32, 4];

        let proof_valid_hashes = [hashes[3], hashes[4]];
        assert!(verify(&hashes, &indices, &proof_valid_hashes).unwrap());

        let proof_invalid_hashes = [
            H256::decode(hex!(
                "0000000000000000000000000000000000000000000000000000000000000006"
            ))
            .unwrap(),
            H256::decode(hex!(
                "0000000000000000000000000000000000000000000000000000000000000007"
            ))
            .unwrap(),
        ];
        assert!(!verify(&hashes, &indices, &proof_invalid_hashes).unwrap());
    }
}
