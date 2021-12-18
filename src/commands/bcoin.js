const { SlashCommandBuilder } = require('@discordjs/builders');
const axios = require('axios').default;

module.exports = {
  data: new SlashCommandBuilder()
    .setName('bcoin')
    .setDescription('Replies with BCOIN token price.'),
  async execute(interaction) {
    const { data } = await axios.get(
      'https://api.coingecko.com/api/v3/coins/bomber-coin'
    );

    const { usd } = data.market_data.current_price;

    console.log(usd);

    return interaction.reply(`Current price: $${usd}`);
  },
};
