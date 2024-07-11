# TODO
* [x] Update
  * [x] Update crate version `winit` 0.28 to 0.30
* [ ] features
  - [ ] Capture range by mouse drag
  - [ ] Draw strings on the image
  - [ ] Floting image window
  - [ ] Paints on the image
    - [ ] Rectangle
    - [ ] Circle
    - [ ] Arrow

もともと capture_and_save_screenshot() においていたものをimpl ApplicationHandler for Appに移動させればよさそう。

1. winitでウィンドウを作成
2. winitでマウスのドラッグ＆ドロップの座標を取得
3. imageで画像を読み込み
4. winitのウィンドウに画像を描画
5. あとは色々できるように
