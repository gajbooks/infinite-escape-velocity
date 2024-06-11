import { Component, Output } from '@angular/core';
import { Subject } from 'rxjs';
import { WebSocketSubject, webSocket } from 'rxjs/webSocket';
import { ClientServerMessage } from 'bindings/ClientServerMessage';
import { ServerClientMessage } from 'bindings/ServerClientMessage';
import { ENVIRONMENT } from 'src/environments/environment';
// @ts-ignore
import * as CBOR from 'cbor-web/dist/cbor';
import { HttpClient } from '@angular/common/http';
import { AssetIndexResponse } from 'bindings/AssetIndexResponse';
import { AssetIndexValue } from 'bindings/AssetIndexValue';

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
  apiBaseUrl = ENVIRONMENT.PRODUCTION ? location.origin : ENVIRONMENT.GAME_SERVER_URL;

  public incomingMessages = new Subject<ServerClientMessage>();
  public outgoingMessages = new Subject<ClientServerMessage>();

  public assetIndex: Promise<AssetIndexValue[]>;

  constructor(private http: HttpClient) {
    let self = this;
    this.assetIndex = new Promise((resolve, reject) => {
      http.get<AssetIndexResponse>(`${this.apiBaseUrl}/assets/index`).subscribe(index => {
        resolve(index.asset_index_list);
        self.subscribeToWebsocket(self);
      });
    })
  }

  disconnectWebsocket() {
    this.outgoingMessages.next({ "type": "Disconnect" });
  }

  subscribeToWebsocket(self: AppComponent) {
    let incomingMessages = self.incomingMessages;
    self.socket.subscribe({
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

    self.outgoingMessages.subscribe(sent => {
      self.socket.next(sent);
    });
  }
}
