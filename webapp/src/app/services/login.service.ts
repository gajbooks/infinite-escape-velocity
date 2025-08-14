import { HttpClient } from "@angular/common/http";
import { inject, Injectable } from "@angular/core";
import { AuthType } from "bindings/players/AuthType";
import { EphemeralPlayerResponse } from "bindings/players/EphemeralPlayerResponse";
import { LoginPlayerResponse } from "bindings/players/LoginPlayerResponse";
import { lastValueFrom } from "rxjs";

@Injectable({
    providedIn: 'root',
})
export class LoginService {
    private client = inject(HttpClient);

    async createEphemeralPlayer(): Promise<string> {
        const request = this.client.post<EphemeralPlayerResponse>('/players/newephemeralplayer', null);
        return await lastValueFrom(request).then(x => x.id);
    }

    async loginEphemeralPlayer(id: string): Promise<string> {
        let auth: AuthType = {BasicToken: { token: id }};
        const request = this.client.post<LoginPlayerResponse>('/players/login', auth);
        return await lastValueFrom(request).then(x => x.session_token);
    }
}