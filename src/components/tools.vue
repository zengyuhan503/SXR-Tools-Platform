<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { EmitRunActions, ListenActiveCloseApp, RunEmitSetLogs } from "../utlis/communication"
import { Modal, message } from 'ant-design-vue';
import { open } from '@tauri-apps/api/dialog';
import { appWindow } from "@tauri-apps/api/window"
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/api/notification';



interface AppItem {
    name: string,
    version: string,
    path: string,
    is_start: boolean,
    install_dir: string,
    text: string,
    loading: boolean
}
interface APPContents {
    apps: string[],
    apps_resources: string,
    contents: string,
    install_json: string
}
interface EmitRunACtionsRes {
    Err: string,
    Ok: string
}

let appList = ref<AppItem[]>([])
let actionTip = ref("卸载中...")
let actionLoading = ref(false);
let isShowProgress = ref(false);
let process_count = ref(5)
let isShowUninstall = ref(false)
let unInstaling = ref(false);
let openCount = 0;



let permissionGranted: boolean | null = null
async function getGranted() {
    isPermissionGranted().then(async res => {
        console.log(res)
        permissionGranted = res;
        console.log(permissionGranted)
        if (!permissionGranted) {
            const permission = await requestPermission();
            permissionGranted = permission === 'granted';
        }
    });

}
getGranted()
/**
 * 监听并处理应用的激活关闭事件
 * 
 * 此函数通过 ListenActiveCloseApp 方法监听应用的激活关闭事件当事件发生时，
 * 根据返回的结果 res 在应用列表 appList 中查找对应的应用如果找到了对应的应用，
 * 将其启动状态 is_start 设置为 false，以标记该应用已被关闭
 * 
 * 注意：appList 应该是一个外部定义的响应式数据源，其内部包含了一系列应用对象，
 * 每个应用对象都有 name 和 is_start 属性，其中 name 表示应用的名称，is_start 表示应用的启动状态
 */
const handleListenActiveCloseApp = () => {
    ListenActiveCloseApp((res) => {
        const app = appList.value.find((app: AppItem) => app.name === res);
        if (app) {
            app.is_start = false;
        }
    })
}

const handleRunActions = (name: string) => {
    return EmitRunActions(name)
}
/**
 * 异步函数：执行安装操作
 * @param {Object} item - 待安装的应用程序项
 * @returns {Promise} - 返回一个Promise对象，代表异步操作的结果
 */
const InstalExe = async (item: AppItem): Promise<any> => {
    process_count.value = 5
    try {
        let install_dir = await open({
            title: "请选择安装目录",
            defaultPath: `D:\\handleFactory\\${item.name}`,
            directory: true
        }) as string
        if (install_dir) {
            const chineseRegex = /[\u4e00-\u9fa5]/;
            let hasZh = chineseRegex.test(install_dir);
            if (hasZh || install_dir.indexOf("C:\\") != -1) {
                Modal.error({
                    title: '应用安装提示',
                    content: '不能选择安装至C盘或安装地址中不能存在中文，请更改路径后重试',
                });
                return false
            }
            if (install_dir == '' || install_dir == null) return false;
            item.text = "安装中..."
            isShowProgress.value = true
            process_count.value = 15
            let res = await EmitRunActions("install", item.install_dir, install_dir) as EmitRunACtionsRes

            if (res.hasOwnProperty("Err")) {
                message.error(res.Err)
            } else if (res.hasOwnProperty("Ok")) {
                setTimeout(() => {
                    process_count.value = 35
                }, 1000);
                setTimeout(() => {
                    process_count.value = 55
                }, 2000);
            }
            setTimeout(() => {
                item.path = res.Ok as string
                actionLoading.value = false
                process_count.value = 100
                setTimeout(() => {
                    isShowProgress.value = false
                    item.text = "安装应用"
                    message.success("安装成功")
                    RunEmitSetLogs("info", `安装${item.name}应用成功，安装目录是${install_dir}`);
                }, 1000);
            }, 5000);
        }
    } catch (error) {
        message.error("安装失败")
        isShowProgress.value = false
        process_count.value = 0
    }
}

const OpenExe = async (item: AppItem) => {
    try {
        item.loading = true;
        let res = await EmitRunActions("open_exe", item.path, item.name) as EmitRunACtionsRes;
        if (res.hasOwnProperty("Err")) {
            message.error(res.Err)
        } else if (res.hasOwnProperty("Ok")) {
            RunEmitSetLogs("info", `打开${item.name}应用成功`);
            item.is_start = true
            openCount++
            if (openCount > 3) {
                if (permissionGranted) {
                    sendNotification({
                        title: 'SXR平台工厂工具提示',
                        body: '你打开的程序已经打开超过3个，可能会影响到测试性能，请注意!',
                    });
                }
            }
            appWindow.minimize()
        }
        item.loading = false
    } catch (error) {
        await GetApps()
    }
}
const CloseExe = (item: AppItem) => {
    EmitRunActions("stop_exe", item.path);
    item.is_start = false
    openCount--
    console.log(openCount)
}
let uninstallItem: AppItem | null = null
const UninstallOk = async () => {
    try {
        unInstaling.value = true
        let item = uninstallItem;
        if (item) {
            let res = await EmitRunActions("uninstall", item.path, item.name) as EmitRunACtionsRes;
            if (res.hasOwnProperty("Err")) {
                message.error(res.Err)
            } else if (res.hasOwnProperty("Ok")) {
                message.success("卸载成功")
                RunEmitSetLogs("info", `卸载${item.name}应用成功`);
                if (uninstallItem) {
                    uninstallItem.path = ""
                    uninstallItem.text = "安装应用"
                }
            }
        }

    } catch (error) {
        await GetApps()
    }
    setTimeout(() => {
        actionLoading.value = false
        unInstaling.value = false
        isShowUninstall.value = false
    }, 2000);
}
/**
 * 异步卸载应用
 * @param {Object} item - 待处理的应用项
 *  - is_start (boolean): 应用是否正在运行的标志
 *  - name (string): 应用的名称
 */
const UnInstallApp = async (item: AppItem) => {
    if (item.is_start) {
        Modal.warning({
            title: "提示",
            content: `${item.name}应用正在运行中，请关闭后再进行卸载`,
        });
    } else {
        uninstallItem = item;
        isShowUninstall.value = true
    }
}
/**
 * 异步获取应用程序列表
 * 此函数通过调用handleRunActions函数初始化应用程序列表，然后更新前端应用列表（appList）
 * 它首先尝试从返回的apps对象中获取已安装的应用程序列表，然后将其与当前应用列表合并
 * 如果在合并过程中发现应用程序已存在，则不会重复添加
 * 如果在初始化过程中发生错误，将捕获异常并打印到控制台
 */
const GetApps = async () => {
    try {
        let apps: APPContents = await handleRunActions("init") as unknown as APPContents;
        if (apps) {
            let contents = apps.contents
            let install_apps = contents !== '' ? JSON.parse(contents) : [];
            install_apps.forEach((item: AppItem) => {
                item.loading = false
            })
            appList.value = [...install_apps];
            apps.apps.forEach(app => {
                let data = app.split("_");
                let exists = install_apps.some((in_app: AppItem) => in_app.name === data[0]);
                if (!exists) {
                    let item: AppItem = {
                        "name": data[0],
                        "version": data[1],
                        "path": "",
                        "is_start": false,
                        "install_dir": app,
                        "text": "安装应用",
                        loading: false
                    }
                    appList.value.push(item)
                }
            });
        }
    } catch (error) {
        console.error("Error initializing apps:", error);
    }
}


defineExpose({ appList, isShowUninstall, isShowProgress, GetApps })
onMounted(async () => {
    await GetApps()
    handleListenActiveCloseApp();
})
</script>
<template>
    <div class="tools">
        <div class="items">
            <a-row :gutter="24">
                <a-col class="gutter-row" :span="8" v-for="(item, index) in appList" :key="index">
                    <a-spin :tip="actionTip" :spinning="actionLoading">
                        <div class="tool">
                            <div class="title">
                                <p>{{ item.name }}</p>
                                <div v-if="item.path != ''">
                                    <a-button size="small" @click="OpenExe(item)" :loading="item.loading"
                                        v-if="!item.is_start" type="primary">打开工具</a-button>
                                    <a-button size="small" @click="CloseExe(item)" v-else danger>关闭工具</a-button>
                                </div>
                                <div v-else>
                                    <a-button size="small" @click="InstalExe(item)">{{ item.text }}</a-button>
                                </div>
                            </div>
                            <div class="body">
                                <img src="../assets/image/tool_icon.png" alt="">
                                <img src="../assets/image/rm.png" @click="UnInstallApp(item)" v-if="item.path != ''"
                                    class="rm_icon" alt="">
                            </div>
                        </div>
                    </a-spin>
                </a-col>
            </a-row>
        </div>
    </div>

    <div class="tool-dialog" v-if="isShowUninstall">
        <div class="dialog-content">
            <div class="dialog-close" @click="isShowUninstall = false">
                <img src="../assets/image/close_icon.png" alt="">
            </div>
            <p class="title">卸载该工具</p>
            <p class="desc">卸载该工具后将不可打开，如需再次使用，请重新下载。</p>
            <a-button type="primary" danger @click="UninstallOk" :loading="unInstaling">卸载工具</a-button>
        </div>
    </div>

    <div class="install-progress" v-if="isShowProgress">
        <div class="progress-content">
            <p class='title'>
                安装中，请勿进行其他操作
            </p>
            <div class="progress-line">
                <p>正在安装中…</p>
                <div class="progress">
                    <div class="progresing" :style="{ width: process_count + '%' }"></div>
                    <span class="count">{{ process_count }}%</span>
                </div>
            </div>
        </div>
    </div>
</template>
<style lang="less" scoped>
.tools {
    height: calc(100% - 56px);
    overflow-y: auto;
    border-radius: 10px;
    overflow-x: hidden;
    scrollbar-width: none;
    /* Firefox */
    margin-top: 25px;

    .items>div>div {
        margin-bottom: 27px;
    }

    .items .tool {
        background: #FFFFFF;
        box-shadow: 0px 2px 20px 0px rgba(31, 39, 50, 0.06);
        border-radius: 10px;
        padding: 15px;
    }

    .tool .title {
        display: flex;
        justify-content: space-between;

        p {
            font-size: 18px;
            line-height: 25px;
            padding-right: 15px;
            font-weight: 600;
            min-height: 50px;
        }

    }

    .tool .body {
        padding: 31px 0 20px 0;
        position: relative;

        img {
            width: 60px;
            margin-left: 30px;
        }

        .rm_icon {
            width: 34px;
            position: absolute;
            bottom: 0;
            right: 0;
            cursor: pointer;
        }
    }

}
</style>
