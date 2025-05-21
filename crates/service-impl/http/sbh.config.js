
import { match } from "@switchboard/http-config"
export default match({
    "/health": "http://localhost:8080/health",
    "/api/v1/": "http://localhost:8080/api/v1/",
    "/api/v1/health": "http://localhost:8080/api/v1/health",
})