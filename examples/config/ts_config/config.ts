// 相对路径引用本地文件
import { createHttpConfig, Instance, Router } from '../../../utils/switchboard-config/src/http/index.ts';
const rootPathMatcher = Router.PathMatch.createInstance("root", [{
    path: "/api/{*path}",
    priority: 1,
    route: "api_handler",
    template: "/{*path}"
}])
const http_config = createHttpConfig().addInstance(rootPathMatcher).build(rootPathMatcher, {
    maxLoop: 3,
});