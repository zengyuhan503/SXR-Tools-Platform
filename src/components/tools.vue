<script setup>
import { ref, onMounted } from 'vue';
import { EmitRunActions, ListenActiveCloseApp, RunEmitSetLogs } from "../utlis/communication"
import { Modal, message } from 'ant-design-vue';
import { app, process } from '@tauri-apps/api';
import { save, open } from '@tauri-apps/api/dialog';

let appList = ref([])

let actionTip = ref("卸载中...")
let actionLoading = ref(false);
let isShowProgress = ref(false);
let process_count = ref(5)
let isShowUninstall = ref(false)
let unInstaling = ref(false);
let openCount = 0;
const handleListenActiveCloseApp = () => {
    ListenActiveCloseApp((res) => {
        const app = appList.value.find(app => app.name === res);
        if (app) {
            app.is_start = false;
        }
    })
}

const handleRunActions = (name) => {
    return EmitRunActions(name)
}

const InstalExe = async (item) => {
    process_count.value = 5
    try {
        let install_dir = await open({
            title: "请选择安装目录",
            defaultPath: `D:\\handleFactory\\${item.name}`,
            directory: true
        })
        if (install_dir) {
            let regText = /^(?:[\u3400-\u4DB5\u4E00-\u9FEA\uFA0E\uFA0F\uFA11\uFA13\uFA14\uFA1F\uFA21\uFA23\uFA24\uFA27-\uFA29]|[\uD840-\uD868\uD86A-\uD86C\uD86F-\uD872\uD874-\uD879][\uDC00-\uDFFF]|\uD869[\uDC00-\uDED6\uDF00-\uDFFF]|\uD86D[\uDC00-\uDF34\uDF40-\uDFFF]|\uD86E[\uDC00-\uDC1D\uDC20-\uDFFF]|\uD873[\uDC00-\uDEA1\uDEB0-\uDFFF]|\uD87A[\uDC00-\uDFE0])+$/
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
            let res = await EmitRunActions("install", item.install_dir, install_dir)
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
                item.path = install_dir
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
        console.log(error)
        message.error("安装失败")
        isShowProgress.value = false
        process_count.value = 0
    }
}

const OpenExe = async (item) => {
    try {
        let res = await EmitRunActions("open_exe", item.path, item.name);
        if (res.hasOwnProperty("Err")) {
            message.error(res.Err)
        } else if (res.hasOwnProperty("Ok")) {
            RunEmitSetLogs("info", `打开${item.name}应用成功`);
            item.is_start = true
            openCount++
            if (openCount > 3) {
                message.warning('你打开的程序已经打开超过3个，可能会影响到测试性能，请注意。')
            }
        }
    } catch (error) {
        await GetApps()
        console.log(error)
    }
}
const CloseExe = (item) => {
    EmitRunActions("stop_exe", item.path);
    item.is_start = false
    openCount--
}
let uninstallItem = null
const UninstallOk = async () => {
    try {
        unInstaling.value = true
        let item = uninstallItem;
        let res = await EmitRunActions("uninstall", item.path, item.name);
        if (res.hasOwnProperty("Err")) {
            message.error(res.Err)
        } else if (res.hasOwnProperty("Ok")) {
            message.success("卸载成功")
            RunEmitSetLogs("info", `卸载${item.name}应用成功`);
            uninstallItem.path = ""
            uninstallItem.text = "安装应用"
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
const UnInstallApp = async (item) => {
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
const GetApps = async () => {
    try {
        let apps = await handleRunActions("init");
        if (apps) {
            let contents = apps.contents
            let install_apps = contents !== '' ? JSON.parse(contents) : [];
            appList.value = [...install_apps];
            apps.apps.forEach(app => {
                let data = app.split("_");
                let exists = install_apps.some(in_app => in_app.name === data[0]);
                if (!exists) {
                    let item = {
                        "name": data[0],
                        "version": data[1],
                        "path": "",
                        "is_start": false,
                        "install_dir": app,
                        "text": "安装应用"
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
                                    <a-button size="small" @click="OpenExe(item)" v-if="!item.is_start"
                                        type="primary">打开工具</a-button>
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
