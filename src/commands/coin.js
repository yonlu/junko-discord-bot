const { SlashCommandBuilder } = require('@discordjs/builders');
const axios = require('axios').default;

module.exports = {
  data: new SlashCommandBuilder()
    .setName('coin')
    .setDescription('Replies with selected token price.')
    .addStringOption((option) =>
      option.setName('input').setDescription('Coin name input')
    ),
  async execute(interaction) {
    const coinName = interaction.options.getString('input');

    const { data } = await axios.get(
      `https://api.coingecko.com/api/v3/coins/${coinName}`
    );

    const { name, symbol } = data;

    const { usd } = data.market_data.current_price;

    return interaction.reply(
      `${symbol.toString().toUpperCase()} (${name}), Current price: $${usd}`
    );
  },
};
