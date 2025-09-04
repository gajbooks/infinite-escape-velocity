import { HTTP_INTERCEPTORS, provideHttpClient, withInterceptorsFromDi } from "@angular/common/http";
import { ApplicationConfig } from "@angular/core";
import { APIClient } from "./services/api-client.service";

export const appConfig: ApplicationConfig = { providers: [provideHttpClient(withInterceptorsFromDi()), {
    provide: HTTP_INTERCEPTORS,
    useClass: APIClient,
    multi: true
  }] };
