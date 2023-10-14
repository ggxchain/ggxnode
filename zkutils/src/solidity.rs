use sailfish::TemplateOnce;
use std::{error::Error, fs, fs::File, io::Write, path::PathBuf};

#[derive(Debug, clap::Subcommand)]
pub enum Commands {
	Verifier(VerifierCMD),
}

#[derive(Debug, clap::Args)]
pub struct VerifierCMD {
	#[arg(required = true)]
	verification_key: PathBuf,
	#[arg(required = false)]
	#[clap(default_value = "verifier.sol")]
	output: PathBuf,
}

impl VerifierCMD {
	pub fn run(&self) -> Result<(), Box<dyn Error>> {
		let b = fs::read(self.verification_key.clone())?;
		let vk: VerificationKey = serde_json::from_slice(&b)?;

		let ctx = TemplateContext {
			vk_alpha_1: vk.vk_alpha_1,
			vk_beta_2: vk.vk_beta_2,
			vk_gamma_2: vk.vk_gamma_2,
			vk_delta_2: vk.vk_delta_2,
			ic: vk.ic,
		};
		let rendered = ctx.render_once()?;

		let mut file = File::create(self.output.clone())?;
		file.write_all(rendered.as_bytes())?;
		Ok(())
	}
}

#[derive(TemplateOnce)]
#[template(path = "verifier_groth16.sol.stpl")]
struct TemplateContext {
	vk_alpha_1: [String; 3],
	vk_beta_2: [[String; 2]; 3],
	vk_gamma_2: [[String; 2]; 3],
	vk_delta_2: [[String; 2]; 3],
	ic: Vec<[String; 3]>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct VerificationKey {
	protocol: String,
	curve: String,
	#[serde(rename = "nPublic")]
	n_public: i128,
	vk_alpha_1: [String; 3],
	vk_beta_2: [[String; 2]; 3],
	vk_gamma_2: [[String; 2]; 3],
	vk_delta_2: [[String; 2]; 3],
	vk_alphabeta_12: [[[String; 2]; 3]; 2],
	#[serde(rename = "IC")]
	ic: Vec<[String; 3]>,
}
