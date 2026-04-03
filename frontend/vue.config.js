const { defineConfig } = require('@vue/cli-service')
module.exports = defineConfig({
  transpileDependencies: true,
  devServer: {
    allowedHosts: 'all',
    client: {
      webSocketURL: {
        protocol: "wss",
        hostname: "unfarming-untethered-flynn.ngrok-free.dev", // твой ngrok домен
        port: 443,
        pathname: "/ws",
      },
    },
  }
})
