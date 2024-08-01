import { UnlistenFn, emit, listen } from "@tauri-apps/api/event"
import { message } from "ant-design-vue"

interface EmitParams {
    name: string,
    item?: string,
    orther?: string
}
interface payloadType {
    [key: string]: string | undefined | object
}

let unlisten: UnlistenFn | null = null
export const EmitRunActions = (name: string, item?: string, orther?: string) => {
    return new Promise(async (resolve, reject) => {
        try {
            unlisten = await listen("run_actions_res", (e) => {
                if (unlisten) unlisten()
                let res = e.payload as payloadType;
                if (res.hasOwnProperty("Err")) {
                    let err = (res.Err) as string;
                    if(err=='') err="用户取消安装"
                    RunEmitSetLogs("error", err);
                    message.error(err)
                    reject(res["Err"])
                } else {
                    resolve(res)
                }
            })
            let params = {
                name: name,
                item: item,
                orther: orther
            }
            await emit("run_actions", params);
        } catch (error) {
            RunEmitSetLogs("error", JSON.stringify(error) as string);
            reject(error)
        }
    })
}

let unActionClose: UnlistenFn | null = null
export const ListenActiveCloseApp = async (callback: (arg0: string) => void) => {
    if (unActionClose) unActionClose();
    unActionClose = await listen("run_close", e => {
        let res = e.payload as string;
        callback(res);
    })

}

export const RunEmitSetLogs = (level: string, message: string) => {

    let params: EmitParams = {
        name: "set_logs",
        item: level,
        orther: message
    }
    emit("run_actions", params);
}