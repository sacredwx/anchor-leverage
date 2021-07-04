import { Coins, MsgExecuteContract, LCDClient, MnemonicKey, StdTx, StdFee, BlockTxBroadcastResult } from '@terra-money/terra.js';
import dotenv from 'dotenv';
import path from 'path';
dotenv.config({
  path: path.resolve(__dirname, '../', '.env')
});
import util from 'util';

(async () => {
  console.log("Setting up accounts and contracts...");
  // Setup an account with key from mnemonic phrases
  const mk1 = new MnemonicKey({
    mnemonic: process.env.MNEMONIC
  });
  // Setup a network provider to connect
  const terra = new LCDClient({
    URL: process.env.TERRA_NODE_URL as string,
    chainID: process.env.TERRA_CHAIN_ID as string,
  });

  // Create a wallet which points to the network provider
  const test1 = terra.wallet(mk1);

  const deposit = new MsgExecuteContract(
    test1.key.accAddress,
    process.env.CONTRACT_ADDRESS as string,
    {
      "deposit": {}
    },
    new Coins({
      "uluna": 100000000
    })
  );

  const interactTx = await test1.createAndSignTx({
    msgs: [deposit],
  }).catch(console.error) as StdTx;

  const interactTxResult = await terra.tx.broadcast(interactTx).catch(console.error) as BlockTxBroadcastResult;

  console.log(interactTxResult);
  if (interactTxResult.raw_log) {
    console.log('events', util.inspect(JSON.parse(interactTxResult.raw_log), { depth: 20 }));
  }
})();