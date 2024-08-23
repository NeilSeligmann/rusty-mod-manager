import { ApplicationState as ApplicationStateRaw, ApplicationConfig, FrontendConfig } from './bindings';

export type ApplicationState = ApplicationStateRaw & {
	application_config: ApplicationConfig;
	frontend_config: FrontendConfig;
}
