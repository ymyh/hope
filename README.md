Rust编写的OpenGL风格软渲API
- 基于重心坐标的三角形绘制
- 深度测试 + 模板测试 + alpha测试
- 混合
- 面剔除以及暴力裁剪
- 模拟GPU以四个像素为基本处理单位
- 可编程渲染管线，片段着色器被分为两个小阶段，其中一个采样纹理用
- 2D纹理和立方体贴图
- 多种纹理插值选项，包括双线性，三线性以及各向异性过滤
- 多线程绘制（配合SDL2使用更佳）

这里有几个例子，在命令行输入命令以查看，它们会在项目根目录输出png图片

`cargo r --example triangle`

`cargo r --example draw_f`

`cargo r --example plane`

`cargo r --example circles`