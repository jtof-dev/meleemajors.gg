var _____WB$wombat$assign$function_____ = function(name) {return (self._wb_wombat && self._wb_wombat.local_init && self._wb_wombat.local_init(name)) || self[name]; };
if (!self.__WB_pmw) { self.__WB_pmw = function(obj) { this.__WB_source = obj; return this; } }
{
  let window = _____WB$wombat$assign$function_____("window");
  let self = _____WB$wombat$assign$function_____("self");
  let document = _____WB$wombat$assign$function_____("document");
  let location = _____WB$wombat$assign$function_____("location");
  let top = _____WB$wombat$assign$function_____("top");
  let parent = _____WB$wombat$assign$function_____("parent");
  let frames = _____WB$wombat$assign$function_____("frames");
  let opener = _____WB$wombat$assign$function_____("opener");

let bg1 = document.getElementById("bg1")
let bg2 = document.getElementById("bg2")
let bg3 = document.getElementById("bg3")
let bg4 = document.getElementById("bg4")
let bg5 = document.getElementById("bg5")
let bg6 = document.getElementById("bg6")
let time = 5000

let slider = function silder(){
    if(bg1.className === "bgImg"
        && bg2.className === "bgImg"
        && bg3.className === "bgImg"
        && bg4.className === "bgImg"
        && bg5.className === "bgImg"
        && bg6.className === "bgImg"){
            bg1.className = "bgImgShow"
        } else if(bg1.className === "bgImgShow"){
            bg1.className = "bgImg"
            bg2.className = "bgImgShow"
        } else if(bg2.className === "bgImgShow"){
            bg2.className = "bgImg"
            bg3.className = "bgImgShow"
        } else if(bg3.className === "bgImgShow"){
            bg3.className = "bgImg"
            bg4.className = "bgImgShow"
        } else if(bg4.className === "bgImgShow"){
            bg4.className = "bgImg"
            bg5.className = "bgImgShow"
        } else if(bg5.className === "bgImgShow"){
            bg5.className = "bgImg"
            bg6.className = "bgImgShow"
        } else if (bg6.className === "bgImgShow"){
            bg6.className = "bgImg"
            bg1.className = "bgImgShow"
        }

        setTimeout("slider()", time)
}

window.onload = slider()

}
/*
     FILE ARCHIVED ON 04:54:16 Dec 02, 2022 AND RETRIEVED FROM THE
     INTERNET ARCHIVE ON 22:42:12 Jun 01, 2024.
     JAVASCRIPT APPENDED BY WAYBACK MACHINE, COPYRIGHT INTERNET ARCHIVE.

     ALL OTHER CONTENT MAY ALSO BE PROTECTED BY COPYRIGHT (17 U.S.C.
     SECTION 108(a)(3)).
*/
/*
playback timings (ms):
  captures_list: 0.684
  exclusion.robots: 0.072
  exclusion.robots.policy: 0.064
  esindex: 0.009
  cdx.remote: 6.043
  LoadShardBlock: 122.575 (3)
  PetaboxLoader3.datanode: 55.544 (4)
  PetaboxLoader3.resolve: 111.228 (3)
  load_resource: 62.847
*/