import Moralis from 'moralis/node';

Moralis.start({ serverUrl: process.env.MORALIS_SERVER_URL, appId: process.env.MORALIS_APP_ID });

export { Moralis };