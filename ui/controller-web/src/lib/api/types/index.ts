export * from './tcp_route';
export * from './bytes';
export * from './control';
export * from './controller';
export * from './descriptor';
export * from './error';
export * from './kernel';
export * from './protocol';
export * from './tls';
export * from './human_readable';
export * from './listener';
export * from './tcp_service';
export * as Http from './http/index';
export * from './time-duration';
import type { Listener } from './listener';
import type { TcpRoute } from './tcp_route';
import type { TcpService } from './tcp_service';
import type { Tls } from './tls';

export type ServiceConfig = {
	tcp_services: Record<string, TcpService>;
	tcp_listeners: Record<string, Listener>;
	tls: Record<string, Tls>;
	tcp_routes: Record<string, TcpRoute>;
};
