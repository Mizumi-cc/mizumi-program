import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { MizumiProgram } from "../target/types/mizumi_program";

describe("mizumi-program", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.MizumiProgram as Program<MizumiProgram>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
