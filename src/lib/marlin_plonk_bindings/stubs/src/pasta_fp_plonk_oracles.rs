use crate::arkworks::{CamlFp, CamlGVesta};
use crate::pasta_fp_plonk_verifier_index::CamlPastaFpPlonkVerifierIndex;
use commitment_dlog::commitment::caml::CamlPolyComm;
use commitment_dlog::commitment::{shift_scalar, PolyComm};
use mina_curves::pasta::{
    fp::Fp,
    vesta::{Affine as GAffine, VestaParameters},
};
use oracle::{
    self,
    poseidon::PlonkSpongeConstants,
    sponge::{DefaultFqSponge, DefaultFrSponge},
    FqSponge,
};
use plonk_circuits::scalars::{caml::CamlRandomOracles, RandomOracles};
use plonk_protocol_dlog::prover::ProverProof;
use plonk_protocol_dlog::{
    index::VerifierIndex as DlogVerifierIndex, prover::caml::CamlProverProof,
};


#[derive(ocaml::IntoValue, ocaml::FromValue)]
pub struct CamlPastaFpPlonkOracles {
    pub o: CamlRandomOracles<CamlFp>,
    pub p_eval: (CamlFp, CamlFp),
    pub opening_prechallenges: Vec<CamlFp>,
    pub digest_before_evaluations: CamlFp,
}

#[ocaml::func]
pub fn caml_pasta_fp_plonk_oracles_create(
    lgr_comm: Vec<CamlPolyComm<CamlGVesta>>, // the bases to commit polynomials
    index: CamlPastaFpPlonkVerifierIndex,    // parameters
    proof: CamlProverProof<CamlGVesta, CamlFp>, // the final proof (contains public elements at the beginning)
) -> CamlPastaFpPlonkOracles {
    let index: DlogVerifierIndex<'_, GAffine> = index.into();
    let proof: DlogProof<GAffine> = proof.into();
    let lgr_comm: Vec<PolyComm<GAffine>> = lgr_comm.into_iter().map(From::from).collect();

    let p_comm = PolyComm::<GAffine>::multi_scalar_mul(
        &lgr_comm
            .iter()
            .take(proof.public.len())
            .map(|x| x)
            .collect(),
        &proof.public.iter().map(|s| -*s).collect(),
    );
    let (mut sponge, digest_before_evaluations, o, _, p_eval, _, _, _, combined_inner_product) =
        proof.oracles::<DefaultFqSponge<VestaParameters, PlonkSpongeConstants>, DefaultFrSponge<Fp, PlonkSpongeConstants>>(&index, &p_comm);

    sponge.absorb_fr(&[shift_scalar(combined_inner_product)]);

    CamlPastaFpPlonkOracles {
        o: o,
        p_eval: (p_eval[0][0], p_eval[1][0]),
        opening_prechallenges: proof
            .proof
            .prechallenges(&mut sponge)
            .into_iter()
            .map(|x| x.0)
            .collect(),
        digest_before_evaluations: digest_before_evaluations,
    }
}

#[ocaml::func]
pub fn caml_pasta_fp_plonk_oracles_dummy() -> CamlRandomOracles<CamlFp> {
    RandomOracles::<Fp>::zero().into()
}

#[ocaml::func]
pub fn caml_pasta_fp_plonk_oracles_deep_copy(
    x: CamlRandomOracles<CamlFp>,
) -> CamlRandomOracles<CamlFp> {
    x
}
