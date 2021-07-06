import { NetworkInfo, WalletProvider, ConnectType, useConnectedWallet, useWallet, WalletStatus } from '@terra-money/wallet-provider';
import { Coins, LCDClient, MsgExecuteContract } from '@terra-money/terra.js';
import React, { useEffect, useMemo, useState } from 'react';
import ReactDOM from 'react-dom';
import Button from '@material-ui/core/Button';
import TextField from '@material-ui/core/TextField';
import './style.css';

const CONTRACT_ADDRESS = 'terra18tqedj5z0ugdfawf23kqjq3sr8xzud5szv5p9l';
const COLLATERAL_AMOUNT = 100000000;
const DENOM = 6;

const mainnet = {
  name: 'mainnet',
  chainID: 'columbus-4',
  lcd: 'https://lcd.terra.dev',
};

const testnet = {
  name: 'testnet',
  chainID: 'tequila-0004',
  lcd: 'https://tequila-lcd.terra.dev',
};

const walletConnectChainIds: Record<number, NetworkInfo> = {
  0: testnet,
  1: mainnet,
};

function App() {
  const {
    status,
    // network,
    // wallets,
    // availableConnectTypes,
    // availableInstallTypes,
    connect,
    // install,
    disconnect,
  } = useWallet();
  console.log(status);
  const connectedWallet = useConnectedWallet();
  const [collateral, setCollateral]=useState(0);

  const query = () => {
    if (connectedWallet && lcd) {
      console.log('query');
      lcd.bank.balance(connectedWallet.walletAddress).then((coins) => {
        setBank(coins.toString());
      });
      lcd.wasm.contractQuery(CONTRACT_ADDRESS, {'collateral': {'contract_addr': CONTRACT_ADDRESS}}).then((response) => {
        // @ts-ignore
        setCollateral(response.balance/10**DENOM); // TODO: Add msgs types
        // @ts-ignore
        console.log(response.balance);
      });
    } else {
      setCollateral(0);
      setBank(null);
    }
  };

  const deposit=async () => {
    const deposit = await connectedWallet?.post({
      msgs: [new MsgExecuteContract(connectedWallet.walletAddress, CONTRACT_ADDRESS, {
        'deposit': {},
      }, new Coins({
        'uluna': COLLATERAL_AMOUNT,
      }))],
    });

    console.log(deposit);
    setTimeout(query, 5000);
  };

  const [bank, setBank] = useState<null | string>();

  const lcd = useMemo(() => {
    if (!connectedWallet) {
      return null;
    }

    return new LCDClient({
      URL: connectedWallet.network.lcd,
      chainID: connectedWallet.network.chainID,
    });
  }, [connectedWallet]);

  useEffect(query, [connectedWallet, lcd]);

  return (
    <div className="App">
      <header className="App-header">
        {status === WalletStatus.WALLET_NOT_CONNECTED && (
          <Button onClick={()=>connect(ConnectType.CHROME_EXTENSION)} color="primary">
            Connect
          </Button>
        )}
        {status === WalletStatus.WALLET_CONNECTED && (
          <Button onClick={()=>disconnect()} color="primary">
            Disconnect {bank}
          </Button>
        )}
        <Button onClick={()=>deposit()} color="primary">
          Deposit {COLLATERAL_AMOUNT}
        </Button>
        <TextField id="outlined-basic" label="Outlined" variant="outlined" aria-readonly="true" value={collateral} />
      </header>
    </div>
  );
}

ReactDOM.render(
    <WalletProvider
      defaultNetwork={testnet}
      walletConnectChainIds={walletConnectChainIds}
    >
      <App />
    </WalletProvider>,
    document.getElementById('root'),
);
