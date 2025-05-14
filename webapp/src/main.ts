import { enableProdMode } from '@angular/core';

import { AppComponent } from './app/app.component';
import { ENVIRONMENT } from './environments/environment';
import { bootstrapApplication } from '@angular/platform-browser';
import { appConfig } from './app/app.config';

if (ENVIRONMENT.PRODUCTION) {
  enableProdMode();
}

bootstrapApplication(AppComponent, appConfig)