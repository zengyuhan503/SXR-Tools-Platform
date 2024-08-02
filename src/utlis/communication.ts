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
type LeverString = "info" | "error" | "warn" | "debug"

let unlisten: UnlistenFn | null = null
/**
 * 异步函数EmitRunActions用于触发运行动作，并处理响应或错误
 * @param name 动作的名称，用于标识动作类型
 * @param item 动作的子项名称，可选，用于进一步细化动作
 * @param orther 额外信息，可选，用于提供附加的上下文
 * @returns 返回一个Promise，解析为动作的响应数据或拒绝为错误信息
 */
export const EmitRunActions = (name: string, item?: string, orther?: string) => {
    return new Promise(async (resolve, reject) => {
        try {
            // 监听特定事件，以获取动作的响应
            unlisten = await listen("run_actions_res", (e) => {
                // 取消监听时的回调处理
                if (unlisten) unlisten()
                let res = e.payload as payloadType;
                // 检查响应是否有错误信息
                if (res.hasOwnProperty("Err")) {
                    let err = (res.Err) as string;
                    // 处理空错误信息的默认提示
                    if(err=='') err="用户取消安装"
                    // 记录错误日志
                    RunEmitSetLogs("error", err);
                    // 显示错误信息
                    message.error(err)
                    // 拒绝Promise，传递错误信息
                    reject(res["Err"])
                } else {
                    // 解析响应数据，完成Promise
                    resolve(res)
                }
            })
            // 构造动作的参数
            let params = {
                name: name,
                item: item,
                orther: orther
            }
            // 触发运行动作
            await emit("run_actions", params);
        } catch (error) {
            // 记录异常错误日志
            RunEmitSetLogs("error", JSON.stringify(error) as string);
            // 拒绝Promise，传递异常错误
            reject(error)
        }
    })
}

let unActionClose: UnlistenFn | null = null
/**
 * 监听活动关闭事件，并在事件发生时调用回调函数
 * 
 * 此函数用于替换之前的活动关闭监听器，以确保不会有多个监听器同时工作
 * 它监听特定事件，当事件发生时，会将事件附带的数据通过回调函数传递出去
 * 
 * @param callback 回调函数，当监听到活动关闭事件时被调用，包含一个字符串参数
 */
export const ListenActiveCloseApp = async (callback: (arg0: string) => void) => {
    // 取消之前的监听器，避免重复监听
    if (unActionClose) unActionClose();
    // 设置新的监听器，监听“run_close”事件
    unActionClose = await listen("run_close", e => {
        // 将事件负载转换为字符串，并通过回调函数传递出去
        let res = e.payload as string;
        callback(res);
    })
}

/**
 * 触发设置日志的事件
 * 
 * 该函数用于封装一个事件的触发，通过指定的日志级别和信息，来实现统一的日志事件处理
 * 主要用于内部触发，以便在不同的地方统一管理日志信息
 * 
 * @param level 日志级别，例如："info" | "error" | "warn" | "debug"，用于区分日志的严重程度
 * @param message 日志信息，用于详细描述日志的具体内容
 */
export const RunEmitSetLogs = (level: LeverString, message: string) => {

    // 创建事件参数对象，用于传递给事件触发函数
    let params: EmitParams = {
        name: "set_logs", // 事件名称，标识这是一个设置日志的事件
        item: level, // 将日志级别作为事件的一个参数
        orther: message // 将日志信息作为事件的另一个参数
    }
    emit("run_actions", params); // 触发名为'run_actions'的事件，并传递参数对象
}