import { Component, Output } from '@angular/core';
import { Subject } from 'rxjs';
import { WebSocketSubject, webSocket } from 'rxjs/webSocket';
import { ClientServerMessage } from 'bindings/ClientServerMessage';
import { ServerClientMessage } from 'bindings/ServerClientMessage';
import { ENVIRONMENT } from 'src/environments/environment';
// @ts-ignore
import * as CBOR from 'cbor-web/dist/cbor';

function generateWebsocket(url: string): WebSocketSubject<unknown> {
  return webSocket({
    url: url,
    binaryType: 'arraybuffer',
    serializer: (val) => {
      return CBOR.encode(val);
    },
    deserializer: (event) => {
      return CBOR.decode(event.data);
    }
  });
}

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.less']
})
export class AppComponent {
  title = 'Infinite Escape Velocity';
  socket = ENVIRONMENT.PRODUCTION ? generateWebsocket('ws://' + location.host + '/ws') : generateWebsocket('ws://' + ENVIRONMENT.GAME_SERVER_HOST + '/ws');

  public incomingMessages = new Subject<ServerClientMessage>();
  public outgoingMessages = new Subject<ClientServerMessage>();

  constructor() {
    let incomingMessages = this.incomingMessages;
    this.socket.subscribe({
      next(value) {
        if ("type" in <any>value) {
          incomingMessages.next(value as ServerClientMessage);
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
