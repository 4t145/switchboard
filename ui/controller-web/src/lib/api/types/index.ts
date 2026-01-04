export * from './tcp_route';
export * from './bytes';
export * from './control';
export * from './controller';
export * from './descriptor';
export * from './error';
export * from './kernel';
export * from './protocol';
export * from './tls';
export * from './listener';
export * from './tcp_service';
import type { Listener } from './listener';
import type { TcpRoute } from './tcp_route';
import type { TcpService } from './tcp_service';
import type { Tls } from './tls';

export type ServiceConfig = {
	tcpServices: Record<string, TcpService>;
	tcpListeners: Record<string, Listener>;
	tls: Record<string, Tls>;
	tcp_routes: Record<string, TcpRoute>;
};
