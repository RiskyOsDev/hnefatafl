const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const webPack = require('webpack');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = {
    entry: './js_src/index.js',
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: 'index.js',
    },
    plugins: [
        new HtmlWebpackPlugin({
            title: "hnefatafl",
        }),
        new WasmPackPlugin({
            crateDirectory: __dirname
        }),
    ],
    mode: 'development',
    experiments: {
        asyncWebAssembly: true
   }
};
