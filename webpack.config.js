const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const webPack = require('webpack');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = {
    entry: './js_src/index.js',
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: 'index.js',
        clean: true,
    },
    plugins: [
        new HtmlWebpackPlugin({
            title: "hnefatafl",
            template: "html_src/index.html",
            filename: "index.html",
            mobile: true,
            meta: {
                charset: 'utf-8',
                viewport: 'width=device-width, initial-scale=1, shrink-to-fit=no',
            }
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
