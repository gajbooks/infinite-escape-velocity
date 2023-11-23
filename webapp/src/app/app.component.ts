import { Component } from '@angular/core';
import { Observable } from 'rxjs';
import {webSocket} from 'rxjs/webSocket';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.less']
})
export class AppComponent {
  title = 'webapp';
  socket = webSocket('ws://localhost:2718/ws');
  constructor() {
    this.socket.subscribe({
      next(value) {
      console.log(value)
    },
  error(err) {console.warn(err)},
  complete() {console.log('disconnected')}
});
  }

  disconnectWebsocket() {
    this.socket.complete();
  }
}
