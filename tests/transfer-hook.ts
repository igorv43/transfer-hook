import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { TransferHook } from '../target/types/burn_program';
import {
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_2022_PROGRAM_ID,
    ExtensionType,
    createInitializeMintInstruction,
    createAssociatedTokenAccountInstruction,
    createMintToInstruction,
    getMintLen,
    getAssociatedTokenAddressSync,
    createTransferCheckedWithTransferHookInstruction,
    createInitializeTransferHookInstruction,
} from '@solana/spl-token';
import {
    Keypair,
    SystemProgram,
    Transaction,
    sendAndConfirmTransaction,
    LAMPORTS_PER_SOL,
} from '@solana/web3.js';
import { expect } from 'chai';

describe('burn-program', () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const program = anchor.workspace.TransferHook as Program<TransferHook>;
    const wallet = provider.wallet as anchor.Wallet;
    const connection = provider.connection;

    const mint = Keypair.generate();
    const decimals = 9;

    const sourceTokenAccount = getAssociatedTokenAddressSync(
        mint.publicKey,
        wallet.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
    );

    const recipient = Keypair.generate();
    const destinationTokenAccount = getAssociatedTokenAddressSync(
        mint.publicKey,
        recipient.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
    );

    const [extraAccountMetaList] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("extra-account-metas"), mint.publicKey.toBuffer()],
        program.programId
    );

    it("Airdrop SOL to recipient", async () => {
        const signature = await connection.requestAirdrop(
            recipient.publicKey,
            LAMPORTS_PER_SOL
        );
        await connection.confirmTransaction(signature);
    });

    it("Create Mint Account with Transfer Hook Extension", async () => {
        const extensions = [ExtensionType.TransferHook];
        const mintLen = getMintLen(extensions);
        const lamports = await connection.getMinimumBalanceForRentExemption(mintLen);

        const transaction = new Transaction().add(
            SystemProgram.createAccount({
                fromPubkey: wallet.publicKey,
                newAccountPubkey: mint.publicKey,
                space: mintLen,
                lamports,
                programId: TOKEN_2022_PROGRAM_ID,
            }),
            createInitializeTransferHookInstruction(
                mint.publicKey,
                wallet.publicKey,
                program.programId,
                TOKEN_2022_PROGRAM_ID
            ),
            createInitializeMintInstruction(
                mint.publicKey,
                decimals,
                wallet.publicKey,
                null,
                TOKEN_2022_PROGRAM_ID
            )
        );

        const txSig = await sendAndConfirmTransaction(
            connection,
            transaction,
            [wallet.payer, mint],
            { skipPreflight: true }
        );
        console.log(`Create Mint Transaction Signature: ${txSig}`);
    });

    it("Create Token Accounts and Mint Tokens", async () => {
        const mintAmount = 1000 * 10 ** decimals;

        const transaction = new Transaction().add(
            createAssociatedTokenAccountInstruction(
                wallet.publicKey,
                sourceTokenAccount,
                wallet.publicKey,
                mint.publicKey,
                TOKEN_2022_PROGRAM_ID,
                ASSOCIATED_TOKEN_PROGRAM_ID
            ),
            createAssociatedTokenAccountInstruction(
                wallet.publicKey,
                destinationTokenAccount,
                recipient.publicKey,
                mint.publicKey,
                TOKEN_2022_PROGRAM_ID,
                ASSOCIATED_TOKEN_PROGRAM_ID
            ),
            createMintToInstruction(
                mint.publicKey,
                sourceTokenAccount,
                wallet.publicKey,
                mintAmount,
                [],
                TOKEN_2022_PROGRAM_ID
            )
        );

        const txSig = await sendAndConfirmTransaction(
            connection,
            transaction,
            [wallet.payer],
            { skipPreflight: true }
        );
        console.log(`Mint Tokens Transaction Signature: ${txSig}`);
    });

    it("Initialize ExtraAccountMetaList Account", async () => {
        const tx = await program.methods
            .initializeExtraAccountMetaList()
            .accounts({
                payer: wallet.publicKey,
                extraAccountMetaList: extraAccountMetaList,
                mint: mint.publicKey,
                tokenProgram: TOKEN_2022_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
            })
            .rpc({ skipPreflight: true });

        console.log("Initialize ExtraAccountMetaList Transaction:", tx);
    });

    it("Transfer tokens with burn fee", async () => {
        const transferAmount = 100 * 10 ** decimals;
        const burnAmount = Math.floor(transferAmount / 10000); // 0.01%

        const beforeSourceBalance = await connection.getTokenAccountBalance(sourceTokenAccount);
        const beforeDestBalance = await connection.getTokenAccountBalance(destinationTokenAccount);

        console.log("Before Source Balance:", beforeSourceBalance.value.amount);
        console.log("Before Destination Balance:", beforeDestBalance.value.amount);

        const transferInstruction = await createTransferCheckedWithTransferHookInstruction(
            connection,
            sourceTokenAccount,
            mint.publicKey,
            destinationTokenAccount,
            wallet.publicKey,
            BigInt(transferAmount),
            decimals,
            [],
            "confirmed",
            TOKEN_2022_PROGRAM_ID
        );

        const transaction = new Transaction().add(transferInstruction);

        const txSig = await sendAndConfirmTransaction(
            connection,
            transaction,
            [wallet.payer],
            { skipPreflight: true }
        );

        console.log("Transfer Transaction:", txSig);

        await new Promise(resolve => setTimeout(resolve, 2000));

        const afterSourceBalance = await connection.getTokenAccountBalance(sourceTokenAccount);
        const afterDestBalance = await connection.getTokenAccountBalance(destinationTokenAccount);

        console.log("After Source Balance:", afterSourceBalance.value.amount);
        console.log("After Destination Balance:", afterDestBalance.value.amount);
        console.log("Expected Burn Amount:", burnAmount);

        expect(Number(afterSourceBalance.value.amount)).to.equal(
            Number(beforeSourceBalance.value.amount) - transferAmount
        );

        expect(Number(afterDestBalance.value.amount)).to.equal(
            Number(beforeDestBalance.value.amount) + transferAmount - burnAmount
        );
    });
});