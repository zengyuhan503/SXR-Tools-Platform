<script setup lang="ts">
import { ref } from 'vue';
import Toos from './components/tools.vue';
import { appWindow } from '@tauri-apps/api/window';
import { EmitRunActions } from '../src/utlis/communication';
import { message } from 'ant-design-vue';
// This starter template is using Vue 3 <script setup> SFCs
// Check out https://vuejs.org/api/sfc-script-setup.html#script-setup
type icon = {
  [key: string]: URL | boolean | string
}
interface HeadIcon {
  [key: number]: icon
}
let toolsRef = ref<InstanceType<typeof Toos> | null>(null)
let head_ioncs = ref({
  1: {
    "normal": new URL("./assets/image/header-icon (1).png", import.meta.url) as unknown as string,
    "hover": new URL("./assets/image/header-icon (4).png", import.meta.url) as unknown as string,
    isHover: false,
    action: () => { appWindow.minimize() }
  },
  2: {
    "normal": new URL("./assets/image/header-icon (2).png", import.meta.url) as unknown as string,
    "hover": new URL("./assets/image/header-icon (5).png", import.meta.url) as unknown as string,
    isHover: false,
    action: () => { appWindow.toggleMaximize() }
  },
  3: {
    "normal": new URL("./assets/image/header-icon (3).png", import.meta.url),
    "hover": new URL("./assets/image/header-icon (6).png", import.meta.url),
    isHover: false,
    action: () => {
      // EmitRunActions("exit")
      if (toolsRef.value.isShowUninstall || toolsRef.value.isShowProgress) {
        message.error("请等安装或者卸载完成后再关闭程序")
        return false
      }
      
      EmitRunActions("exit")
      setTimeout(() => {
        toolsRef.value.GetApps()
      }, 1000)
    }
  },
} as unknown as HeadIcon)

const getIcon = (item: icon): string => {
  let url = (item.isHover ? item.hover : item.normal) as URL;
  return url.href;
}
const hoverEffect = (item: icon) => {
  item.isHover = !item.isHover;
}
</script>

<template>
  <div class="app-header customization-page" data-tauri-drag-region>
    <div class="title-bar">
      <img src="./assets/image/app-icon.png" class="logo" alt="">
      <span class="title">SXR平台工厂工具</span>
    </div>
    <div class="action-icons">
      <span v-for="(item, index) in head_ioncs" :key="index">
        <img :src="getIcon(item)" @click="item.action" @mouseover="hoverEffect(item)" @mouseleave="hoverEffect(item)"
          alt="">
      </span>
    </div>
  </div>
  <div class="app-body">
    <p class="title">请选择工具</p>
    <Toos ref="toolsRef" />
  </div>

</template>

<style>
@import url("./assets/css/normalize.css");
</style>
<style>
html,
body,
#app,
.container {
  width: 100%;
  height: 100%;
  margin: 0;
  padding: 5px
    /* background: #F1F1F1; */
}
</style>
<style lang="less">
.app-header {
  height: 60px;
  padding: 0 40px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  box-shadow: 0px 7px 7px 0px rgba(0, 0, 0, 0.1);
  background: #fff;
  border-radius: 20px 20px 0 0;
  position: relative;
  z-index: 10;


  .title-bar {
    display: flex;
    align-items: center;
  }

  .logo {
    width: 32px;
    margin-right: 15px;
  }

  .title {
    font-size: 20px;
  }

  .action-icons {
    span {
      img {
        width: 40px;
        cursor: pointer;
        // opacity: 0.5;

        &:hover {
          opacity: 1;
        }
      }

      margin-left: 7px;
    }
  }
}

.app-body {
  width: 100%;
  height: calc(100% - 50px);
  background: #F1F1F1;
  border-radius: 0 0 20px 20px;
  position: relative;
  top: 0px;
  padding: 20px 40px;
  padding-top: 30px;
  box-shadow: 0px 0px 7px 0px rgba(0, 0, 0, 0.3);
  position: relative;
  overflow-y: auto;

  &>.title {
    font-size: 24px;
    color: #1F2732;
  }
}

.tool-dialog {
  position: absolute;
  left: 0;
  top: 0;
  width: 100%;
  height: 100%;
  display: flex;
  justify-content: center;
  align-items: center;

  .dialog-content {
    width: 512px;
    height: 259px;
    background: #FFFFFF;
    box-shadow: 0px 2px 20px 0px rgba(0, 0, 0, 0.1);
    border-radius: 20px;
    position: relative;
    text-align: center;

    .title {
      font-family: PingFangSC, PingFang SC;
      font-weight: 500;
      font-size: 24px;
      color: rgba(0, 0, 0, 0.85);
      line-height: 33px;
      text-align: center;
      margin-top: 40px;
    }

    .desc {
      font-family: PingFangSC, PingFang SC;
      font-weight: 400;
      font-size: 16px;
      color: rgba(0, 0, 0, 0.45);
      line-height: 24px;
      font-style: normal;
      margin-top: 31px;
      // opacity: 0.45;
    }

    button {
      background-color: #FF5059;
      margin-top: 70px;
    }
  }

  .dialog-close {
    position: absolute;
    width: 32px;
    height: 32px;
    top: 24px;
    right: 24px;

    img {
      width: 100%;
      cursor: pointer;

      &:hover {
        opacity: 0.58;
      }
    }
  }

}

.install-progress {
  position: absolute;
  left: 0;
  top: 0;
  width: 100%;
  height: 100%;
  display: flex;
  justify-content: center;
  align-items: center;

  .progress-content {
    width: 512px;
    height: 259px;
    background: #FFFFFF;
    box-shadow: 0px 2px 20px 0px rgba(0, 0, 0, 0.1);
    border-radius: 20px;
    padding: 40px 28px;

    .title {
      font-family: PingFangSC, PingFang SC;
      font-weight: 500;
      font-size: 24px;
      color: rgba(0, 0, 0, 0.85);
      line-height: 33px;
      text-align: center;
      font-style: normal;
    }

    .progress-line {
      margin-top: 32px;

      &>p {
        font-family: PingFangSC, PingFang SC;
        font-weight: 400;
        font-size: 16px;
        color: #000000;
        line-height: 22px;
        text-align: left;
        font-style: normal;
      }

      .progress {
        margin-top: 15px;
        height: 40px;
        border-radius: 8px;
        border: 1px dashed rgba(0, 0, 0, 0.08);
        position: relative;
        width: 455px;
        padding: 1px;
        overflow: hidden;

        .progresing {
          position: absolute;
          left: 1px;
          top: 1px;
          background: rgba(95, 162, 240, 1);
          width: 50%;
          height: 38px;
          z-index: 99;
          transition: width 1s;
        }

        .count {
          font-family: PingFangSC, PingFang SC;
          font-weight: 400;
          font-size: 16px;
          color: rgba(0, 0, 0, 0.88);
          line-height: 24px;
          font-style: normal;
          position: absolute;
          z-index: 999;
          height: 40px;
          top: 0;
          left: 16px;
          line-height: 40px;
        }
      }
    }
  }
}
</style>
