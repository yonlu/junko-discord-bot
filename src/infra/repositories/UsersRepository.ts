import { prismaClient } from '../../shared/prismaClient';

async function main() {
	await prismaClient.user.create({
		data: {
			discord_id: 'USER_DISCORD_ID',
			wallet: 'USER_WALLET',
		}
	});

	const allUsers = await prismaClient.user.findMany();

	console.dir(allUsers, { depth: null });
}

main();