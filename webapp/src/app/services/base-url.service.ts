import { Injectable } from "@angular/core";
import { ENVIRONMENT } from 'src/environments/environment';

@Injectable({
    providedIn: 'root',
})
export class BaseUrlService {
    getGameServerHost(): string {
        return ENVIRONMENT.PRODUCTION ? location.host : ENVIRONMENT.GAME_SERVER_HOST;
    }

    getApiBaseUrl(): string {
        return ENVIRONMENT.PRODUCTION ? location.origin : ENVIRONMENT.GAME_SERVER_URL;
    }

    generateWebsocketUrl(): string {
      let prefix = location.protocol == 'https:' ? 'wss://' : 'ws://';
      let host = this.getGameServerHost();
      return `${prefix}${host}/ws`;
    }
}
