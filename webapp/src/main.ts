import { enableProdMode } from '@angular/core';

import { bootstrapApplication } from '@angular/platform-browser';

import { AppComponent } from './app/app.component';
import { ENVIRONMENT } from './environments/environment';
import { appConfig } from './app/app.config';

if (ENVIRONMENT.PRODUCTION) {
  enableProdMode();
}


bootstrapApplication(AppComponent, appConfig)
  .catch((err) => console.error(err))
