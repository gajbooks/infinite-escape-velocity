import { NgModule } from '@angular/core';
import { BrowserModule } from '@angular/platform-browser';

import { AppComponent } from './app.component';
import { GameplayCanvasComponent } from './gameplay-canvas/gameplay-canvas.component';

@NgModule({
  declarations: [
    AppComponent
  ],
  imports: [
    BrowserModule,
    GameplayCanvasComponent
  ],
  providers: [],
  bootstrap: [AppComponent]
})
export class AppModule { }
