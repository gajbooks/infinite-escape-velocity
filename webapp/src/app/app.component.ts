import { Component, Output } from '@angular/core';
import { Subject } from 'rxjs';
import { webSocket } from 'rxjs/webSocket';
import { ClientServerMessage } from 'bindings/ClientServerMessage';
import { ServerClientMessage } from 'bindings/ServerClientMessage';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.less']
})
export class AppComponent {
  title = 'Infinite Escape Velocity';
  socket = webSocket('ws://localhost:2718/ws');

  public incomingMessages = new Subject<ServerClientMessage>();
  public outgoingMessages = new Subject<ClientServerMessage>();

  constructor() {
    let incomingMessages = this.incomingMessages;
    this.socket.subscribe({
      next(value) {
        if ("type" in <any>value) {
          incomingMessages.next(<ServerClientMessage>value);
        } else {
          console.warn("Received garbage type from server: ", value);
        }
      },
      error(err) { console.warn(err) },
      complete() { console.log('disconnected') }
    });

    this.outgoingMessages.subscribe(sent => {
      this.socket.next(sent);
    });
  }

  disconnectWebsocket() {
    this.outgoingMessages.next({ "type": "Disconnect" });
  }
}
