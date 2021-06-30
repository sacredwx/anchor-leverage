const { LCDClient, MnemonicKey } = require('@terra-money/terra.js');

import { Coins, MsgInstantiateContract, MsgStoreCode } from '@terra-money/terra.js';
import dotenv from 'dotenv';
import path from 'path';
dotenv.config({
  path: path.resolve(__dirname, '../', '.env')
});

const CODE_ID = null; // Define if already deployed

async function deploy() {
  console.log("Setting up accounts and contracts...");
  // Setup an account with key from mnemonic phrases
  const mk1 = new MnemonicKey({
    mnemonic: process.env.MNEMONIC
  });
  // Setup a network provider to connect
  const terra = new LCDClient({
    URL: process.env.TERRA_NODE_URL,
    chainID: process.env.TERRA_CHAIN_ID,
  });

  // Create a wallet which points to the network provider
  const test1 = terra.wallet(mk1);

  const fs = require("fs");
  // To deploy a contract, you must get an wasm file.
  const code = fs.readFileSync("../contracts/anchor-leverage/artifacts/anchor_leverage.wasm");

  // Create tx to sign and send to the blockchain
  const store = new MsgStoreCode(
    test1.key.accAddress,
    code.toString("base64")
  );

  // Create a batch of txs to send to the blockchain
  const storeCodeTx = await test1
    .createAndSignTx({
      msgs: [store],
    })
    .catch((error: any) => {
      console.log(error);
    });

  // Get results with codeId
  let codeId = null;
  if (!CODE_ID) {
    const storeCodeTxResult = await terra.tx.broadcast(storeCodeTx);
    codeId = storeCodeTxResult.logs[0].events[1].attributes[1].value;
  } else {
    codeId = CODE_ID;
  }

  console.log("Code id", codeId);

  // Cosmwasm smart contracts need to instantiate from the uploaded wasmer executable binary.
  const instantiate = new MsgInstantiateContract(
    test1.key.accAddress,
    +codeId,
    {
      config: {
        bluna_hub_contract: "terra1fflas6wv4snv8lsda9knvq2w0cyt493r8puh2e",
        bluna_token_contract: "terra1ltnkx0mv7lf2rca9f8w740ashu93ujughy4s7p",
        bluna_collateral_contract: "terra1u0t35drzyy0mujj8rkdyzhe264uls4ug3wdp3x",
        anchor_overseer_contract: "terra1qljxd0y3j3gk97025qvl3lgq8ygup4gsksvaxv",
        anchor_market_contract: "terra15dwd5mj8v59wpj0wvt233mf5efdff808c5tkal",
        terraswap_luna_ust: "terra156v8s539wtz0sjpn8y8a8lfg8fhmwa7fy22aff",
        preferred_validator: "terravaloper1krj7amhhagjnyg2tkkuh6l0550y733jnjnnlzy",
      }
    },
    new Coins({}),
    false
  );

  console.log(instantiate);

  // Create tx batch again
  const instantiateTx = await test1.createAndSignTx({
    msgs: [instantiate],
  }).catch((error: any) => {
    console.log(error);
  });

  console.log(instantiateTx);

  // Get address from executing tx
  const instantiateTxResult = await terra.tx.broadcast(instantiateTx).catch((error: any) => {
    console.log(error);
  });

  console.log(instantiateTxResult);

  const contractAddress =
    instantiateTxResult.logs[0].events[0].attributes[2].value;

  console.log("address", contractAddress);
}

deploy();
