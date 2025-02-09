// utils.ts
import * as anchor from '@coral-xyz/anchor';
import { 
    PublicKey, 
    Keypair, 
    Connection,
    sendAndConfirmTransaction,
    Transaction
} from '@solana/web3.js';
import { 
    TOKEN_PROGRAM_ID,
    MINT_SIZE,
    createInitializeMintInstruction,
    getMinimumBalanceForRentExemptMint,
    getAssociatedTokenAddress,
    createAssociatedTokenAccountInstruction,
    createMintToInstruction
} from '@solana/spl-token';
import { ContributionTracker } from '../target/types/contribution_tracker';

export const EPOCH_DURATION = 3; // 3 seconds for testing
export const MIN_POINTS_THRESHOLD = 100;
export const TOKENS_PER_PERIOD = 10_000;

export async function createTestMint(
    provider: anchor.AnchorProvider
): Promise<PublicKey> {
    try {
        // Generate mint keypair
        const mintKeypair = Keypair.generate();
        console.log("Created mint keypair");

        // Get rent and create transaction
        const lamports = await getMinimumBalanceForRentExemptMint(provider.connection);
        console.log("Got minimum balance for mint:", lamports);

        const transaction = new Transaction().add(
            // Create account
            anchor.web3.SystemProgram.createAccount({
                fromPubkey: provider.wallet.publicKey,
                newAccountPubkey: mintKeypair.publicKey,
                space: MINT_SIZE,
                lamports,
                programId: TOKEN_PROGRAM_ID,
            }),
            // Initialize mint
            createInitializeMintInstruction(
                mintKeypair.publicKey,
                9,
                provider.wallet.publicKey,
                provider.wallet.publicKey,
                TOKEN_PROGRAM_ID
            )
        );

        console.log("Sending create mint transaction...");
        await sendAndConfirmTransaction(
            provider.connection,
            transaction,
            [provider.wallet.payer, mintKeypair],
            { commitment: 'confirmed' }
        );
        console.log("Mint created successfully");

        return mintKeypair.publicKey;
    } catch (error) {
        console.error("Error in createTestMint:", error);
        throw error;
    }
}

export async function setupTokenVaults(
    provider: anchor.AnchorProvider,
    mint: PublicKey
): Promise<{ rewardVault: PublicKey, reserveVault: PublicKey }> {
    try {
        console.log("Setting up token vaults...");

        // Create ATAs
        const rewardVault = await getAssociatedTokenAddress(
            mint,
            provider.wallet.publicKey
        );
        console.log("Reward vault address derived:", rewardVault.toBase58());

        // Create token account if it doesn't exist
        try {
            const transaction = new Transaction().add(
                createAssociatedTokenAccountInstruction(
                    provider.wallet.publicKey,  // payer
                    rewardVault,                // ata
                    provider.wallet.publicKey,  // owner
                    mint                        // mint
                )
            );

            await sendAndConfirmTransaction(
                provider.connection,
                transaction,
                [provider.wallet.payer]
            );
            console.log("Reward vault created");
        } catch (e) {
            // Account might already exist, which is fine
            console.log("Reward vault might already exist:", e.message);
        }

        // Create reserve vault
        const reserveVault = await getAssociatedTokenAddress(
            mint,
            provider.wallet.publicKey
        );
        console.log("Reserve vault address derived:", reserveVault.toBase58());

        try {
            const transaction = new Transaction().add(
                createAssociatedTokenAccountInstruction(
                    provider.wallet.publicKey,  // payer
                    reserveVault,               // ata
                    provider.wallet.publicKey,  // owner
                    mint                        // mint
                )
            );

            await sendAndConfirmTransaction(
                provider.connection,
                transaction,
                [provider.wallet.payer]
            );
            console.log("Reserve vault created");
        } catch (e) {
            // Account might already exist, which is fine
            console.log("Reserve vault might already exist:", e.message);
        }

        // Mint tokens to reward vault
        try {
            const mintTx = new Transaction().add(
                createMintToInstruction(
                    mint,                       // mint
                    rewardVault,                // destination
                    provider.wallet.publicKey,  // authority
                    1_000_000                   // amount
                )
            );

            await sendAndConfirmTransaction(
                provider.connection,
                mintTx,
                [provider.wallet.payer]
            );
            console.log("Tokens minted to reward vault");
        } catch (e) {
            console.error("Error minting tokens:", e);
            throw e;
        }

        return { rewardVault, reserveVault };
    } catch (error) {
        console.error("Error in setupTokenVaults:", error);
        throw error;
    }
}

export const waitForTransaction = async (
    connection: Connection,
    signature: string,
): Promise<void> => {
    console.log("Waiting for transaction:", signature);
    
    const latestBlockhash = await connection.getLatestBlockhash();
    
    await connection.confirmTransaction({
        signature,
        blockhash: latestBlockhash.blockhash,
        lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
    }, 'confirmed');
    
    console.log("Transaction confirmed:", signature);
};

export const sleep = (ms: number): Promise<void> => {
    return new Promise(resolve => setTimeout(resolve, ms));
};

export const getCurrentPeriodPda = async (
    program: anchor.Program<ContributionTracker>,
    tracker: any
): Promise<PublicKey> => {
    const [periodPda] = PublicKey.findProgramAddressSync(
        [
            Buffer.from("distribution_period"),
            new anchor.BN(tracker.currentPeriod).toArrayLike(Buffer, 'le', 8)
        ],
        program.programId
    );
    return periodPda;
};