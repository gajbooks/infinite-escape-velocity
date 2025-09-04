import { inject, Injectable } from "@angular/core";
import { LoginService } from "./login.service";

const SESSION_TOKEN_NAME = "SESSION_TOKEN";

@Injectable({
    providedIn: 'root',
})
export class SessionService {
    private session = inject(LoginService);

    private current_session_token: string | null = null;

    private session_await: null | Promise<string> = null;

    async getCurrentSessionToken(): Promise<string> {
        if (this.current_session_token != null) {
            return this.current_session_token;
        } else {
            if (this.session_await == null) {
                this.session_await = new Promise(async (resolve) => {
                    let session_token = sessionStorage.getItem(SESSION_TOKEN_NAME);

                    let reaquire = true;

                    if (session_token != null) {
                        reaquire = !this.session.loginTokenIsGood(session_token);
                    }

                    if (reaquire || session_token == null) {
                        let ephemeral_player = await this.session.createEphemeralPlayer();
                        session_token = await this.session.loginEphemeralPlayer(ephemeral_player);

                        sessionStorage.setItem(SESSION_TOKEN_NAME, session_token);
                    }

                    resolve(session_token);
                });
            }

            this.current_session_token = await this.session_await;
            return this.current_session_token;
        }
    }
}
