import { BigNumber, utils, Contract, providers } from 'ethers';

import { Moralis } from '../shared/moralis';
import { result as abi } from './abi.json';

const provider = new providers.WebSocketProvider('wss://speedy-nodes-nyc.moralis.io/41b78a236d663145680d8658/bsc/mainnet/ws');

async function getNativeBalance(address: string) {
	const { balance } = await Moralis.Web3API.account.getNativeBalance({ chain: 'bsc', address });
	const parsedBalance = BigNumber.from(balance);
	const value = utils.formatUnits(parsedBalance);
	const formattedValue = value.slice(0, 6);
	return formattedValue;
}

async function getBcoinBalance(address: string) {
	const contract = new Contract('0x00e1656e45f18ec6747f5a8496fd39b50b38396d', abi, provider);
	const balance = await contract.functions.balanceOf(address)
		.then(value => {
			return utils.formatUnits(value.toString(), 'ether').slice(0, 6);
		});
	return balance;
}

export { getNativeBalance, getBcoinBalance };