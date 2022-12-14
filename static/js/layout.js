socket = null;
hb = null;
musicPaused = false;
songDuration = 0;
playlistSort = "timestamp";
playlistSortDir = 1;
loadedPlaylistArtwork = false;

function sendHB(){
    socket.send(JSON.stringify({type:"hb",data:{}}));
}

function setUpWebSocketConnection(){
    socket = new WebSocket("wss://wss.lolstat.net")
    socket.onopen = function (event){
        console.info("Connected to WS");
        hb = setInterval(sendHB,5000);
    }

    socket.onmessage = function (event){
        var data = JSON.parse(event.data)
        //console.log(data)
        if(data){
            if(data.type == "trackUpdate"){
                if(data.song) {
                    if(data.song.title.length > 32) {
                        $(".activeTrack").html("<marquee>" + data.song.title + "</marquee>");
                    }
                    else{
                        $(".activeTrack").html(data.song.title);
                    }
                    $(".activeArtist").html(data.song.artist.name || "");
                    if (data.song.artwork) {
                        $(".activeArtwork").attr("style", "background: url('" + data.song.artwork + "') no-repeat center; background-size: cover;");
                    }
                    else {
                        $(".activeArtwork").attr("style", "");
                    }
                    if ($("#playlist").length) {
                        if($("#playlist").attr("data-playlistId") == data.song.playlistId) {
                            $(".trackPlaying").removeClass("trackPlaying");
                            $(".play").parent().html($(".play").parent().attr('data-trackNo'));
                            $("#" + data.song.songId + " .item").html($("#" + data.song.songId + " .item").html()+'<i class="fa fa-play play" aria-hidden="true"></i>');
                            $("#" + data.song.songId).addClass("trackPlaying");
                        }
                    }
                    if($("#currentSong").length) {
                        $(".currentSongListItem").remove();
                        $("li[data-songId='" + data.song.songId + "']").each(function(index){
                            if(index == 0){
                                $(this).remove();
                            }
                        });
                        $("#currentSong").html("<div class='artwork' style=\"background: url('"+data.song.artwork+"') no-repeat center; background-size: cover;\"></div><div class='title'>"+data.song.title+"</div><div class='album_name'>"+(data.song.album.name||"")+"</div><div class='artist_name'>"+(data.song.artist.name || "")+"</div><div class='length'>"+secondsToHms(data.song.duration)+"</div>");
                    }
                    songDuration = data.song.duration;
                }
            }
            else if(data.type == "songDone"){
                $("#playStop").html('<i class="fa fa-play" aria-hidden="true" onclick="resumeMusic();" style="cursor: pointer;""></i>');
                $(".playPlaylistButton").html('<i class="fa fa-play" aria-hidden="true"></i> &nbsp; &nbsp;PLAY').click(function(event){playMusic()})
                if ($("#currentSong").length) {
                    $(".currentSongListItem").remove();
                }
                $(".activeTrack").html("");
                $(".activeArtist").html("");
                $(".activeArtwork").attr("style", "");
            }
            else if(data.type == "playUpdate"){
                if(data.status == "stop"){
                    $("#playStop").html('<i class="fa fa-play" aria-hidden="true" onclick="playMusic();" style="cursor: pointer;""></i>')
                    $(".playPlaylistButton").html('<i class="fa fa-play" aria-hidden="true"></i> &nbsp; &nbsp;PLAY').click(function(event){playMusic()})
                }
                else if (data.status == "pause") {
                    $("#playStop").html('<i class="fa fa-play" aria-hidden="true" onclick="resumeMusic();" style="cursor: pointer;""></i>')
                    $(".playPlaylistButton").html('<i class="fa fa-play" aria-hidden="true"></i> &nbsp; &nbsp;PLAY').click(function(event){resumeMusic()})
                }
                else if(data.status == "play"){
                    $("#playStop").html('<i class="fa fa-pause" aria-hidden="true" onclick="pauseMusic();" style="cursor: pointer;""></i>')
                    $(".playPlaylistButton").html('<i class="fa fa-pause" aria-hidden="true"></i> &nbsp; &nbsp;PAUSE').click(function(event){pauseMusic()})
                }
            }
            else if(data.type == "trackAdded"){
                //new chrome extension
                console.log(data);
                if($("#playlist").length) {
                    if($("#playlist").attr("data-playlistId") == data.playlistId){
                        var trackNo = $("#playlist li").length;
                        var formattedTimestamp = convertTimestamp(data.duration)
                        var added = "a few seconds ago";
                        $("#playlist").append("<li id='" + data._id + "' data-songId='" + data._id + "' data-playlistId='" + data.playlistId + "'><div class='trackRow'><div class='item' data-trackNo='" + trackNo + "'>" + trackNo + "</div><div class='title'>" + data.title + "</div><div class='artist'>"+data.artist+"</div><div class='album'>"+data.album+"</div><div class='timestamp'>" + added + "</div><div class='time'>" + formattedTimestamp + "</div></div></li>");
                        $(".errorList ul").append("<li class='info'><div class='icon'><div class='iconWrapper'><i class='fa fa-play' aria-hidden='true'></i></div></div><div class='content'>Track Added: \"" + data.title + "\"</div></li>");
                        if (data.albumArt != "" && !loadedPlaylistArtwork) {
                            $(".header .bg").attr("style", "background: url('" + data.albumArt + "') no-repeat center; background-size: cover;");
                            $(".header .artwork").attr("style", "background: url('" + data.albumArt + "') no-repeat center; background-size: cover;");
                            loadedPlaylistArtwork = true;
                        }
                        updateDblClicks();
                    }
                    else{
                        console.log("New Track Added, not in current playlist though");
                    }
                }
            }
            else if(data.type == "trackDelete"){
                if($("#playlist").length && $("#playlist").attr("data-playlistId") == data.playlistId){
                    $("#" + data.songId).remove();
                    $("#playlist li").each(function (i) {
                        if (i > 0 && !$(this).hasClass("trackPlaying")) {
                            $(this).children('.item').html(i);
                        }
                    });
                    if (data.newAlbumArt) {
                        $(".header .bg").attr("style", "background: url('" + data.newAlbumArt + "') no-repeat center; background-size: cover;");
                        $(".header .artwork").attr("style", "background: url('" + data.newAlbumArt + "') no-repeat center; background-size: cover;");
                        loadedPlaylistArtwork = true;
                    }
                    else if(data.newAlbumArt == ""){
                        $(".header .bg").attr("style", "");
                        $(".header .artwork").attr("style", "");
                        loadedPlaylistArtwork = false;
                    }
                }
            }
            else if(data.type == "songTime"){
                var timeSpent = new Date(data.time).toISOString().substr(11, 8);
                var songTimelineDuration = new Date(songDuration*1000).toISOString().substr(11, 8)
                if(timeSpent.substr(0,3) == "00:"){
                    timeSpent = timeSpent.substr(3,5);
                }
                if(songTimelineDuration.substr(0,3) == "00:"){
                    songTimelineDuration = songTimelineDuration.substr(3,5);
                }
                $(".duration").html(timeSpent);
                $(".songDuration").html(songTimelineDuration);
                var newWidth = ((data.time/1000)/songDuration)*100;
                if(newWidth <= 100) {
                    $(".timeline").animate({
                        "width": newWidth + "%"
                    }, 1000, "linear");
                }
            }
            else if(data.type == "voiceUpdate"){
                if(data.status == "join") {
                    $("#selectedChannel").html(data.channel).removeClass("yellow").addClass("green")
                }
                else{
                    $("#selectedChannel").html("Disconnected").removeClass("green").addClass("yellow")
                }
            }
            else if(data.type == "randomUpdate"){
                if (data.status) {
                    $(".random").addClass("active");
                    if ($("#nextSongsList").length) {
                        $("#nextSongsList > li:not(.titleRow)").sort(function dec_sort(a, b) {
                            return ($(b).attr("data-randId")) > ($(a).attr("data-randId")) ? 1 : -1;
                        }).appendTo('#nextSongsList');
                    }
                }
                else{
                    $(".random").removeClass("active");
                    if ($("#nextSongsList").length) {
                        $("#nextSongsList > li:not(.titleRow)").sort(function dec_sort(a, b) {
                            return ($(b).attr("data-sortId")) > ($(a).attr("data-sortId")) ? 1 : -1;
                        }).appendTo('#nextSongsList');
                    }
                }
            }
            else{
            }
        }
    }

    socket.onclose = function (event){
        console.error("Websocket Closed...")
        $("#socketClose").css("display","block");
    }

    window.addEventListener('beforeunload', function(event) {
        clearInterval(hb);
        console.info("WebSocket About to Unload");
    });
}

function loadView(view,param){
    console.log(view+"  "+param);
    var url = "";
    if(param == "undefined"){
        param = undefined
        url = "/views/"+view;
        window.history.pushState('MotorBot Dashboard View', view, 'https://motorbot.io/dashboard/'+view);
    }
    else{
        url = "/views/"+view+"/"+param;
        window.history.pushState('MotorBot Dashboard View', view, 'https://motorbot.io/dashboard/'+view+'/'+param);
    }
    $("#mainNavBar").find("li.active").removeClass("active");
    $("#mainNavBar").find("li[data-view='"+view+"']").addClass("active");
    $(".contentView").html("");
    $(".loader").css("display","block");
    $("#subTitleBar").css("display","none");
    $.ajax({
        url: url,
        dataType: "html",
        success: function(data){
            $(".contentView").html(data);
            $(".loader").css("display", "none");
            if(view == "playlists" && param){
                $(".contentView").css("display","none");
                $(".loader").css("display","block");
                $("#playlist").attr("data-playlistId",param);
                loadPlaylist(param);
                $(".contentView").scroll(function () {
                    if ($(".contentView").scrollTop() > 160) {
                        $(".header").css("position", "fixed").css("height","100px");
                        $("#playlist").find(".titleRow").css("position","fixed").css("top","120px");
                        $(".playlistType").css("display","none");
                        $(".playlistName").css("top","26px").css("font-size","25px");
                        $(".playlistOptions").css("top","21px").css("left","calc(50% + 204px)").css("text-align","right");
                    }
                    else {
                        $(".header").css("position", "absolute").css("height","250px");
                        $("#playlist").find(".titleRow").css("position","absolute").css("top","272px");
                        $(".playlistType").css("display","block");
                        $(".playlistName").css("top","70px").css("font-size","38px");
                        $(".playlistOptions").css("top","160px").css("left","calc(50% - 564px)").css("text-align","left");
                    }
                });
            }
            else if(view == "playlists"){
                $.ajax({
                    method: "GET",
                    url: "/api/user/playlists?api_key=caf07b8b-366e-44ab-9bda-623f94a9c2df",
                    dataType: 'json',
                    beforeSend: function (xhr) {
                        xhr.setRequestHeader("Authorization", "Bearer #{user.motorbotAccessToken}");
                    },
                    success: function(data){
                        $(".contentView").off("scroll");
                        $("#subTitleBar").css("display","block");
                        $.each(data, function (i, item) {
                            artwork = "";
                            if(item.artwork != ""){
                                artwork = "background: url(\""+item.artwork+"\") no-repeat center; background-size: cover;";
                            }
                            $(".playlistList").append("<li onClick=\"loadView('playlists','"+item.id+"')\"><div class='artwork' style='"+artwork+"'><div class='playState'><i class=\"fa fa-play\" aria-hidden=\"true\"></i></div></div><div class='playlistName'>"+item.name+"</div><div class='creator'><div class='username'>"+item.creatorName.username+"#"+item.creatorName.discriminator+"</div></div></li>")
                        });
                    },
                    error: function(err){
                        console.log(err);
                    }
                });
            }
            else if (view == "browse") {
                $.ajax({
                    method: "GET",
                    url: "/api/browse?api_key=caf07b8b-366e-44ab-9bda-623f94a9c2df",
                    dataType: 'json',
                    beforeSend: function (xhr) {
                        xhr.setRequestHeader("Authorization", "Bearer #{user.motorbotAccessToken}");
                    },
                    success: function (data) {
                        $(".contentView").off("scroll");
                        $.each(data.spotlight, function (i, item) {
                            artwork = "";
                            if (item.artwork != "") {
                                artwork = "background: url(\"" + item.artwork + "\") no-repeat center; background-size: cover;";
                            }
                            $(".playlistList#spotlight").append("<li onClick=\"loadView('playlists','" + item.id + "')\"><div class='artwork' style='" + artwork + "'><div class='playState'><i class=\"fa fa-play\" aria-hidden=\"true\"></i></div></div><div class='playlistName' style='height: 40px; white-space: normal;'>" + item.name + "</div></li>")
                        });
                        $.each(data.heavy_rotation, function (i, item) {
                            item = item.playlist
                            artwork = "";
                            if (item.artwork != "") {
                                artwork = "background: url(\"" + item.artwork + "\") no-repeat center; background-size: cover;";
                            }
                            $(".playlistList#on_heavy_rotation").append("<li onClick=\"loadView('playlists','" + item.id + "')\"><div class='artwork' style='" + artwork + "'><div class='playState'><i class=\"fa fa-play\" aria-hidden=\"true\"></i></div></div><div class='playlistName' style='height: 40px; white-space: normal;'>" + item.name + "</div></li>")
                        });
                    },
                    error: function (err) {
                        console.log(err);
                    }
                });
            }
            else if(view == "home"){
                $.ajax({
                    url: "https://api.github.com/repos/motorlatitude/motorbot/commits",
                    dataType: "json",
                    success: function(data){
                        console.log(data);
                        $.each(data, function (i, item) {
                            $("#commitHistory").append("<li><div class='date'>"+item.commit.author.date.replace(/T/gmi,"<br/>").replace(/Z/gmi,"")+"</div><div class='commit'>"+item.sha.substr(0,7)+"</div><div class='author'>"+item.author.login+"</div><div class='container'>"+marked(item.commit.message.replace(/<(.*?)>$/gmi,"&lt;$1&gt;").replace(/FIX/g,"<div class='type fix'>FIX</div>").replace(/NEW/g,"<div class='type new'>NEW</div>").replace(/TODO/g,"<div class='type todo'>TODO</div>").replace(/CODE\sIMPROVEMENT/g,"<div class='type code'>CODE</div>").replace(/CODE/g,"<div class='type code'>CODE</div>").replace(/IMPROVEMENT/g,"<div class='type improvement'>IMPROVEMENT</div>").replace(/DEPRECIATED/g,"<div class='type depreciated'>DEPRECIATED</div>"))+"</div></li>");
                        });
                    },
                    error: function(err){
                        console.log(err);
                    }
                })
            }
            else if (view == "account") {

            }
            else if (view == "queue") {
                $(".contentView").css("display", "none");
                $(".loader").css("display", "block");
                $.ajax({
                    url: "/api/queue?api_key=caf07b8b-366e-44ab-9bda-623f94a9c2df",
                    dataType: "json",
                    success: function (data) {
                        var k = 0, x = 0, m=0, s=1;
                        $.each(data, function (i, item) {
                            console.log(item.status)
                            if(item.status == "added") {
                                m++
                                $("#nextSongsList").append("<li id='" + item._id + "' data-songId='" + item.songId + "' data-playlistId='" + item.playlistId + "' data-sortId='"+item.sortId+"' data-randId='"+item.randId+"'><div class='trackRow'><div class='item'>"+s+"</div><div class='title'>" + item.title + "</div><div class='artist'>" + (item.artist.name || "") + "</div><div class='album'>" + (item.album.name || "") + "</div><div class='time'>" + secondsToHms(item.duration) + "</div></div></li>");
                                s++
                            }
                            else if (item.status == "playing") {
                                x++
                                $("#currentSong").html("<div class='artwork' style=\"background-color: rgba(0,0,0,0.2); background: url('"+item.artwork+"') no-repeat center; background-size: cover;\"></div><div class='title'>"+item.title+"</div><div class='album_name'>"+(item.album.name || "")+"</div><div class='artist_name'>"+(item.artist.name || "")+"</div><div class='length'>"+secondsToHms(item.duration)+"</div>");
                            }
                            else if (item.status == "queued") {
                                k++
                                $("#queueList").append("<li id='" + item._id + "' data-songId='" + item.songId + "' data-playlistId='" + item.playlistId + "'><div class='trackRow'><div class='item'>"+s+"</div><div class='title'>" + item.title + "</div><div class='artist'>" + item.artist.name + "</div><div class='album'>" + item.album.name + "</div><div class='time'>" + secondsToHms(item.duration) + "</div></div></li>");
                                s++
                            }
                        });
                        if(k == 0){
                            $("#queueList").parent(".playlistQueueContainer").css("display","none");
                        }
                        else {
                            $("#nextSongsList").parent(".playlistQueueContainer").css("margin-top", "50px");
                        }
                        if (m == 0) {
                            $("#nextSongsList").parent(".playlistQueueContainer").css("display", "none");
                        }
                        if (x == 0) {
                            $("#currentSong").html("<div class='artwork'></div><div class='title'></div><div class='album_name'>Nothing Currently Playing</div><div class='artist_name'></div><div class='length'></div>");
                        }
                        $(".contentView").css("display", "block");
                        $(".loader").css("display", "none");
                    },
                    error: function (err) {
                        console.log(err);
                    }
                })
            }
        },
        error: function(err){
            console.log(err);
        }
    })
}

function playSongFromPlaylist(songId, playlistId){
    $.ajax({
        url: '/api/music/play/song?id=' + songId + '&playlist_id=' + playlistId + '&sort=' + playlistSort + '&sort_dir=' + playlistSortDir + '&api_key=caf07b8b-366e-44ab-9bda-623f94a9c2df',
        dataType: 'json',
        success: function (d) {
            console.log(d);
            $(".trackPlaying").removeClass("trackPlaying");
            $(".play").parent().html($(".play").parent().attr('data-trackNo'))
            self.children('.item').html(self.children('.item').html()+'<i class="fa fa-play play" aria-hidden="true"></i>');
            self.addClass('trackPlaying');
            songDuration = 0
        },
        error: function (err) {
            console.log(err);
        }
    });
}

$(document).ready(function(){
    document.oncontextmenu = function(){return false;};
    if (electron_app) {
        console.log("module object")
        $(".electronBar").css("display","block");
        $(".titleBar").css("top", "34px");
        $(".bodyWrapper").css("top", "34px");
        $(".errorList").css("top","+=34px");
    }
    $(document).click(function(e){
        console.log(e.target);
        if(e.target.className != "playlistExtraOptions" && e.target.className != "fa fa-ellipsis-h"){
            $(".contextMenu").css("display", "none");
        }
    });
    setUpWebSocketConnection();
    $(".errorList ul li").each(function(index){
        $(this).click(function(){
            $(this).remove();
        });
    });
    loadView("#{view}","#{param}");
    loadPlaying();
    loadRandom();
    getCurrentChannel();
    $("#mainNavBar li").each(function(index){
        $(this).click(function(){
            loadView($(this).attr("data-view"),"undefined");
        });
    });
    $(".newPlaylistButton").click(function () {
        $(".contentView").css("-webkit-filter", "blur(5px)");
        $(".modalityOverlay").css("display", "block");
        $(".modalFrame.newPlaylist").css("display", "block");
    });
    $(".importPlaylistButton").click(function (){
        $("#importPlaylistLoader").css("display","block");
        $(".contentView").css("-webkit-filter", "blur(5px)");
        $(".modalityOverlay").css("display", "block");
        $(".modalFrame.importPlaylist").css("display", "block");
        $.ajax({
            method: "GET",
            url: "/api/spotify/playlists",
            dataType: "json",
            success: function(data){
                $.each(data, function(i, playlist){
                    $("#spotifyPlaylists").append("<option value='"+playlist.id+"' data-ownerid='"+playlist.owner.id+"'>"+playlist.name+"</option>");
                });
                $("#importPlaylistLoader").css("display","none");
            },
            error: function(err){

            }
        })
    });
    $("#newPlaylistConfirm").click(function () {
        var newPlaylistName = $("#newPlaylistName").val()
        console.log("New Playlist Confirm")
        if (newPlaylistName && newPlaylistName != "") {
            //ajax
            $.ajax({
                method: "POST",
                url: "/api/playlist?api_key=caf07b8b-366e-44ab-9bda-623f94a9c2df",
                dataType: "json",
                processData: false,
                contentType: 'application/json',
                data: JSON.stringify({"playlist_name": newPlaylistName}),
                beforeSend: function (xhr) {
                    xhr.setRequestHeader("Authorization", "Bearer #{user.motorbotAccessToken}");
                },
                success: function (data) {
                    loadView("playlists", "undefined");
                    $(".modalityOverlay").css("display", "none");
                    $(".modalFrame.newPlaylist").css("display", "none");
                    $(".contentView").css("-webkit-filter", "blur(0px)");
                },
                error: function (err) {
                    console.log(err);
                }
            });
        }
        else {
            //no name given :(
            alert("Please give your new playlist a name.");
        }
    });
    $("#addToPlaylist_confirmNewPlaylistName").click(function(e){
        var newPlaylistName = $("#addToPlaylist_newPlaylistName").val()
        var songId = $("#addToPlaylist_confirmNewPlaylistName").attr("data-songId")
        if (newPlaylistName && newPlaylistName != "" &&  songId != "") {
            //ajax
            $.ajax({
                url: "/api/addSongToNewPlaylist/"+songId+"/" + encodeURIComponent(newPlaylistName),
                dataType: "json",
                success: function (data) {
                    $(".modalityOverlay").css("display", "none");
                    $(".modalFrame.addToPlaylist").css("display", "none");
                },
                error: function (err) {
                    console.log(err);
                }
            });
        }
        else {
            //no name given :(
            alert("Please give your new playlist a name.");
        }
    });
    $("#importPlaylistConfirm").click(function(e){
        console.log("Importing playlist");
        $("#importPlaylistLoader").css("display","block");
        var spotify_playlist = $("#spotifyPlaylists").val();
        var owner_id = $("#spotifyPlaylists option[value='"+spotify_playlist+"']").attr("data-ownerid");
        if(spotify_playlist && owner_id) {
            $.ajax({
                method: "PUT",
                url: "/api/spotify/playlist/" + spotify_playlist + "/owner/"+owner_id+"?api_key=caf07b8b-366e-44ab-9bda-623f94a9c2df",
                dataType: "json",
                timeout: 0,
                beforeSend: function (xhr) {
                    xhr.setRequestHeader("Authorization", "Bearer #{user.motorbotAccessToken}");
                },
                success: function (data) {
                    loadView("playlists", "undefined");
                    $(".modalityOverlay").css("display", "none");
                    $(".modalFrame.importPlaylist").css("display", "none");
                    $(".contentView").css("-webkit-filter", "blur(0px)");
                    $("#importPlaylistLoader").css("display","none");
                    if(data.not_found){
                        var not_found = ""
                        $.each(data.not_found, function(i, item){
                            not_found += item+"\n";
                        });
                        if(not_found != "") {
                            alert("Sorry, we didn't find the following songs:\n" + not_found);
                        }
                    }
                },
                error: function (err) {
                    console.log(err);
                }
            });
        }
        else{
            console.log("spotify_playlist:"+spotify_playlist);
            console.log("owner_id:"+owner_id);
        }
    });
    $(document).keydown(function (e) {
        console.log(document.activeElement.tagName.toLowerCase());
        if(document.activeElement.tagName.toLowerCase() != "input") {
            switch (e.which) {
                case 38: // up
                    if ($("#playlist").length > 0) {
                        var prev = $("#playlist").find("li:not(.titleRow).active").prev();
                        if (prev.length > 0 && !prev.hasClass("titleRow")) {
                            prev.next().removeClass("active")
                            prev.addClass("active");
                            if (prev.offset().top < 140) {
                                $(".contentView").scrollTop($(".contentView").scrollTop() - 29);
                            }
                        }
                    }
                    break;
                case 40: // down
                    if ($("#playlist").length > 0) {
                        var next = $("#playlist").find("li:not(.titleRow).active").next();
                        if (next.length > 0) {
                            next.prev().removeClass("active")
                            next.addClass("active");
                            if (($(document).height() - next.offset().top) < 85) {
                                $(".contentView").scrollTop($(".contentView").scrollTop() + 29);
                            }
                        }
                    }
                    break;
                case 13: //enter
                    if ($("#playlist").length > 0) {
                        var songId = $("#playlist").find("li:not(.titleRow).active").attr("data-songId");
                        var playlistId = $("#playlist").find("li:not(.titleRow).active").attr("data-playlistId");
                        playSongFromPlaylist(songId, playlistId)
                    }
                case 32: //space
                    break;
                case 8: //backspace
                    var songId = $("#playlist").find("li:not(.titleRow).active").attr("data-songId");
                    var playlistId = $("#playlist").find("li:not(.titleRow).active").attr("data-playlistId");
                    if ($("#playlist").length > 0) {
                        var next = $("#playlist").find("li:not(.titleRow).active").next();
                        if (next.length > 0) {
                            next.prev().removeClass("active")
                            next.addClass("active");
                            if (($(document).height() - next.offset().top) < 85) {
                                $(".contentView").scrollTop($(".contentView").scrollTop() + 29);
                            }
                        }
                    }
                    deleteSongFromPlaylist(songId, playlistId);
                    break;
                default:
                    console.log(e.which)
                    return; // exit this handler for other keys
            }
            e.preventDefault(); // prevent the default action (scroll / move caret)
        }
    });
});

function updateDblClicks(){
    $(".errorList ul li").each(function(index){
        $(this).click(function(){
            $(this).remove();
        });
    });
    $("#playlist li:not(.titleRow)").each(function(index){
        var self = $(this);
        self.click(function(){
            $("#playlist").find("li:not(.titleRow).active").removeClass("active");
            self.addClass("active");
        });
        self.dblclick(function(){
            if (musicPaused) {
                console.log("Stopping Music First");
                resumeMusic();
                stopMusic();
            }
            playSongFromPlaylist(self.attr('data-songId'), self.attr('data-playlistId'))
        });
        self.on('contextmenu', function(e){
            $(".contextMenu").css("display","none")
            if( e.button == 2 ) {
                CurX = (window.Event) ? e.pageX : e.clientX + (document.documentElement.scrollLeft ? document.documentElement.scrollLeft : document.body.scrollLeft);
                CurY = (window.Event) ? e.pageY : e.clientY + (document.documentElement.scrollTop ? document.documentElement.scrollTop : document.body.scrollTop);
                $(".contextMenu").html("<ul></ul>");
                if(CurY+200 > $(document).height()) {
                    $(".contextMenu").css("display", "block").css("top", (CurY - 190) + "px").css("left", CurX + "px");
                }
                else{
                    $(".contextMenu").css("display", "block").css("top", CurY + "px").css("left", CurX + "px");
                }
                if($(this).attr("id")){
                    if($("#playlist").attr("data-playlistCreator") == "#{user.id}"){
                        $(".contextMenu ul").append("<li onclick=\"addSongToQueue('"+$(this).attr("data-songId")+"','"+$(this).attr("data-playlistId")+"')\">Add to Queue</li><li class='sep'></li><li disabled='true'>Add to Library</li><li onclick=\"addToPlaylist('"+$(this).attr("data-songId")+"')\">Add to Playlist</li><li class='sep'></li><li onclick=\"deleteSongFromPlaylist('"+$(this).attr("data-songId")+"','"+$(this).attr("data-playlistId")+"')\" style='color: rgba(242, 52, 51, 1.00);'>Remove This From Playlist</li>");
                    }
                    else{
                        $(".contextMenu ul").append("<li onclick=\"addSongToQueue('"+$(this).attr("data-songId")+"','"+$(this).attr("data-playlistId")+"')\">Add to Queue</li><li class='sep'></li><li disabled='true'>Add to Library</li><li onclick=\"addToPlaylist('"+$(this).attr("data-songId")+"')\">Add to Playlist</li>")
                    }
                }
                return false;
            }
            return true;
        });
    });
    $(".titleRow .timestamp").click(function(){
        if (playlistSortDir == 1) {
            $(".titleRow .sortDir").remove();
            $(".titleRow .timestamp").append("<span class='sortDir' data-sortdir='-1' data-sort='timestamp'><i class=\"fa fa-chevron-up\" aria-hidden=\"true\"></i></div>");
            $("#playlist > li:not(.titleRow)").sort(function dec_sort(a, b) {
                return ($(b).find(".timestamp").attr("data-sortIndex")) > ($(a).find(".timestamp").attr("data-sortIndex")) ? 1 : -1;
            }).appendTo('#playlist');
            $("#playlist > li:not(.titleRow)").each(function (index) {
                track_no = index + 1
                $(this).find(".trackRow .item").attr("data-trackno", track_no).html(track_no)
            });
            playlistSortDir = -1;
            playlistSort = "timestamp";
        }
        else if (playlistSortDir == -1) {
            $(".titleRow .sortDir").remove();
            $(".titleRow .timestamp").append("<span class='sortDir' data-sortdir='1' data-sort='timestamp'><i class=\"fa fa-chevron-down\" aria-hidden=\"true\"></i></div>");
            $("#playlist > li:not(.titleRow)").sort(function dec_sort(a, b) {
                return ($(b).find(".timestamp").attr("data-sortIndex")) < ($(a).find(".timestamp").attr("data-sortIndex")) ? 1 : -1;
            }).appendTo('#playlist');
            $("#playlist > li:not(.titleRow)").each(function (index) {
                track_no = index + 1
                $(this).find(".trackRow .item").attr("data-trackno", track_no).html(track_no)
            });
            playlistSortDir = 1;
            playlistSort = "timestamp";
        }
    });
    $(".titleRow .title").click(function () {
        if (playlistSortDir == 1) {
            $(".titleRow .sortDir").remove();
            $(".titleRow .title").append("<span class='sortDir' data-sortdir='-1' data-sort='title'><i class=\"fa fa-chevron-up\" aria-hidden=\"true\"></i></div>");
            $("#playlist > li:not(.titleRow)").sort(function dec_sort(a, b) {
                return ($(b).find(".title").attr("data-sortIndex")) > ($(a).find(".title").attr("data-sortIndex")) ? 1 : -1;
            }).appendTo('#playlist');
            $("#playlist > li:not(.titleRow)").each(function (index) {
                track_no = index + 1
                $(this).find(".trackRow .item").attr("data-trackno", track_no).html(track_no)
            });
            playlistSortDir = -1;
            playlistSort = "title";
        }
        else if (playlistSortDir == -1) {
            $(".titleRow .sortDir").remove();
            $(".titleRow .title").append("<span class='sortDir' data-sortdir='1' data-sort='title'><i class=\"fa fa-chevron-down\" aria-hidden=\"true\"></i></div>");
            $("#playlist > li:not(.titleRow)").sort(function dec_sort(a, b) {
                return ($(b).find(".title").attr("data-sortIndex")) < ($(a).find(".title").attr("data-sortIndex")) ? 1 : -1;
            }).appendTo('#playlist');
            $("#playlist > li:not(.titleRow)").each(function (index) {
                track_no = index + 1
                $(this).find(".trackRow .item").attr("data-trackno", track_no).html(track_no)
            });
            playlistSortDir = 1;
            playlistSort = "title";
        }
    });
    $(".titleRow .artist").click(function () {
        if (playlistSortDir == 1) {
            $(".titleRow .sortDir").remove();
            $(".titleRow .artist").append("<span class='sortDir' data-sortdir='-1' data-sort='artist'><i class=\"fa fa-chevron-up\" aria-hidden=\"true\"></i></div>");
            $("#playlist > li:not(.titleRow)").sort(function dec_sort(a, b) {
                return ($(b).find(".artist").attr("data-sortIndex")) > ($(a).find(".artist").attr("data-sortIndex")) ? 1 : -1;
            }).appendTo('#playlist');
            $("#playlist > li:not(.titleRow)").each(function (index) {
                track_no = index + 1
                $(this).find(".trackRow .item").attr("data-trackno", track_no).html(track_no)
            });
            playlistSortDir = -1;
            playlistSort = "artist";
        }
        else if (playlistSortDir == -1) {
            $(".titleRow .sortDir").remove();
            $(".titleRow .artist").append("<span class='sortDir' data-sortdir='1' data-sort='artist'><i class=\"fa fa-chevron-down\" aria-hidden=\"true\"></i></div>");
            $("#playlist > li:not(.titleRow)").sort(function dec_sort(a, b) {
                return ($(b).find(".artist").attr("data-sortIndex")) < ($(a).find(".artist").attr("data-sortIndex")) ? 1 : -1;
            }).appendTo('#playlist');
            $("#playlist > li:not(.titleRow)").each(function (index) {
                track_no = index + 1
                $(this).find(".trackRow .item").attr("data-trackno", track_no).html(track_no)
            });
            playlistSortDir = 1;
            playlistSort = "artist";
        }
    });
    $(".titleRow .album").click(function () {
        if (playlistSortDir == 1) {
            $(".titleRow .sortDir").remove();
            $(".titleRow .album").append("<span class='sortDir' data-sortdir='-1' data-sort='album'><i class=\"fa fa-chevron-up\" aria-hidden=\"true\"></i></div>");
            $("#playlist > li:not(.titleRow)").sort(function dec_sort(a, b) {
                return ($(b).find(".album").attr("data-sortIndex")) > ($(a).find(".album").attr("data-sortIndex")) ? 1 : -1;
            }).appendTo('#playlist');
            $("#playlist > li:not(.titleRow)").each(function (index) {
                track_no = index + 1
                $(this).find(".trackRow .item").attr("data-trackno", track_no).html(track_no)
            });
            playlistSortDir = -1;
            playlistSort = "album";
        }
        else if (playlistSortDir == -1) {
            $(".titleRow .sortDir").remove();
            $(".titleRow .album").append("<span class='sortDir' data-sortdir='1' data-sort='album'><i class=\"fa fa-chevron-down\" aria-hidden=\"true\"></i></div>");
            $("#playlist > li:not(.titleRow)").sort(function dec_sort(a, b) {
                return ($(b).find(".album").attr("data-sortIndex")) < ($(a).find(".album").attr("data-sortIndex")) ? 1 : -1;
            }).appendTo('#playlist');
            $("#playlist > li:not(.titleRow)").each(function (index) {
                track_no = index + 1
                $(this).find(".trackRow .item").attr("data-trackno", track_no).html(track_no)
            });
            playlistSortDir = 1;
            playlistSort = "album";
        }
    });
}

function getCurrentChannel(){
    $.ajax({
        url: "/api/motorbot/channel?api_key=caf07b8b-366e-44ab-9bda-623f94a9c2df",
        dataType: "json",
        success: function (data) {
            if (data.channel) {
                $("#selectedChannel").html(data.channel).removeClass("yellow").addClass("green")
            }
            else {
                $("#selectedChannel").html("Disconnected").removeClass("green").addClass("yellow")
            }
        },
        error: function (err) {
            console.log(err);
        }
    });
}

function addSongToQueue(songId, playlistId){
    $.ajax({
        method: "PUT",
        url: "/api/queue/song/" + songId + "/playlist/" + playlistId + "?api_key=caf07b8b-366e-44ab-9bda-623f94a9c2df",
        dataType: "json",
        success: function (data) {

        },
        error: function (err) {
            console.log(err);
        }
    });
}

function addSongToThisPlaylist(songId, playlistId){
    $.ajax({
        method: "PATCH",
        url: "/api/playlist/"+playlistId+"/song/"+songId+"?api_key=caf07b8b-366e-44ab-9bda-623f94a9c2df",
        dataType: "json",
        beforeSend: function (xhr) {
            xhr.setRequestHeader("Authorization", "Bearer #{user.motorbotAccessToken}");
        },
        success: function (data){
            $(".modalityOverlay").css("display", "none");
            $(".modalFrame.addToPlaylist").css("display", "none");
        },
        error: function(err){
            console.log(err);
        }
    });
}

function addToPlaylist(songId){
    $(".modalityOverlay").css("display", "block");
    $(".modalFrame.addToPlaylist").css("display","block");
    $(".modal_playlistList").html("");
    $("#addToPlaylist_confirmNewPlaylistName").attr("data-songId","");
    $("#addToPlaylist_newPlaylistName").val("");
    $.ajax({
        url: "/api/user/playlists?api_key=caf07b8b-366e-44ab-9bda-623f94a9c2df",
        dataType: 'json',
        beforeSend: function (xhr) {
            xhr.setRequestHeader("Authorization", "Bearer #{user.motorbotAccessToken}");
        },
        success: function (data) {
            $.each(data, function (i, item) {
                if(item.creator.toString() == "#{user.id}") {
                    artwork = "";
                    if (item.artwork != "") {
                        artwork = "background: url(\"" + item.artwork + "\") no-repeat center; background-size: cover;";
                    }
                    $(".modal_playlistList").append("<li onClick=\"addSongToThisPlaylist('"+songId+"','"+item.id+"')\"><div class='artwork' style='" + artwork + "'></div><div class='playlistName'>" + item.name + "</div></li>")
                    $("#addToPlaylist_confirmNewPlaylistName").attr("data-songId",songId)
                }
            });
        },
        error: function (err) {
            console.log(err);
        }
    });

}

function deleteSongFromPlaylist(songId,playlistId){
    $.ajax({
        method: "DELETE",
        url: "/api/playlist/"+playlistId+"/song/"+songId+"?api_key=caf07b8b-366e-44ab-9bda-623f94a9c2df",
        dataType: "json",
        beforeSend: function (xhr) {
            xhr.setRequestHeader("Authorization", "Bearer #{user.motorbotAccessToken}");
        },
        success: function(data){
            console.log(data)
        },
        error: function(err){
            console.log(err);
        }
    });
}

function convertTimestamp(input){
    var reptms = /^PT(?:(\d+)H)?(?:(\d+)M)?(?:(\d+)S)?$/
    var hours = 0
    var minutes = 0
    var seconds = 0

    if(reptms.test(input)){
        matches = reptms.exec(input)
        if (matches[1]) hours = Number(matches[1])
        if (matches[2]) minutes = Number(matches[2])
        if (matches[3]) seconds = Number(matches[3])
        if (minutes < 10) minutes = "0"+minutes
        if (seconds < 10) seconds = "0"+seconds
    }
    if(hours == 0){
        return minutes+" : "+seconds
    }
    else{
        return hours+" : "+minutes+" : "+seconds
    }
}

function convertTimestampToSeconds(input){
    var reptms = /^PT(?:(\d+)H)?(?:(\d+)M)?(?:(\d+)S)?$/
    var hours = 0
    var minutes = 0
    var seconds = 0

    if(reptms.test(input)){
        matches = reptms.exec(input)
        if (matches[1]) hours = Number(matches[1])
        if (matches[2]) minutes = Number(matches[2])
        if (matches[3]) seconds = Number(matches[3])
    }
    return hours*60*60+minutes*60+seconds;
}

function millisecondsToStr(timestamp){
    var diff = moment.unix(timestamp/1000).fromNow();
    return diff;
}

function secondsToHms(d) {
    d = Number(d);
    var h = Math.floor(d / 3600);
    var m = Math.floor(d % 3600 / 60);
    var s = Math.floor(d % 3600 % 60);
    return ((h > 0 ? h + ":" + (m < 10 ? "0" : "") : "") + m + ":" + (s < 10 ? "0" : "") + s);
}


function loadPlaylist(playlistID){
    loadedPlaylistArtwork = false
    var totalDuration = 0;
    $.ajax({
        method: "GET",
        url: "/api/playlist/"+playlistID+"?api_key=caf07b8b-366e-44ab-9bda-623f94a9c2df",
        dataType: 'json',
        beforeSend: function (xhr) {
            xhr.setRequestHeader("Authorization", "Bearer #{user.motorbotAccessToken}");
        },
        success: function(d){
            if(d.artwork){
                $(".header .bg").attr("style", "background: url('" + d.artwork + "') no-repeat center; background-size: cover;");
                $(".header .artwork").attr("style", "background: url('" + d.artwork + "') no-repeat center; background-size: cover;");
                loadedPlaylistArtwork = true;
            }
            $("#playlist").attr("data-playlistCreator", d.creator);
            $.each(d.songs, function(i, item){
                if(item != null) {
                    var data = d.songs[i];
                    var trackNo = i + 1;
                    var songDuration = data.duration;
                    var formattedTimestamp = secondsToHms(data.duration)
                    totalDuration += songDuration
                    var added = millisecondsToStr(data.date_added);
                    var user_id = d.user_id || "";
                    if (data.artwork != "" && !loadedPlaylistArtwork) {
                        console.log("Using Artwork for track: " + i);
                        $(".header .bg").attr("style", "background: url('" + data.artwork + "') no-repeat center; background-size: cover;");
                        $(".header .artwork").attr("style", "background: url('" + data.artwork + "') no-repeat center; background-size: cover;");
                        loadedPlaylistArtwork = true;
                    }
                    artist = data.artist.name || ""
                    album = data.album.name || ""
                    $("#playlist").append("<li id='" + data.id + "' data-songId='" + data.id + "' data-playlistId='" + playlistID + "'><div class='trackRow'><div class='item' data-trackNo='" + trackNo + "'>" + trackNo + "</div><div class='title' data-sortIndex='" + data.title.toUpperCase() + "'>" + data.title + "</div><div class='artist' data-sortIndex='" + artist.toUpperCase() + "'>" + artist + "</div><div class='album' data-sortIndex='" + album.toUpperCase() + "'>" + album + "</div><div class='timestamp' data-sortIndex='" + data.date_added + "'>" + added + "</div><div class='time'>" + formattedTimestamp + "</div></div></li>");
                }
                else{
                    console.log(i)
                }
            });
            $(".contentView").css("display","block");
            $(".loader").css("display","none");
            $(".playlistName").html(d.name);
            $(".playlistStats .user").html(d.creatorName.username+"#"+d.creatorName.discriminator);
            if((d.followers.length - 1) == 1) {
                $(".playlistStats .followCount").html((d.followers.length - 1) + " Follower");
            }
            else if((d.followers.length - 1) < 0) {
                $(".playlistStats .followCount").html("0 Followers");
            }
            else{
                $(".playlistStats .followCount").html((d.followers.length - 1) + " Followers");
            }
            $(".songTotal").html(d.songs.length);
            if("#{user.id}" == d.creator.toString()){
                $(".header .followPlaylistButton").css("display","none");
            }
            var hrs = Math.floor((totalDuration/60)/60)
            var mins = Math.round((((totalDuration/60)/60) - hrs)*60);
            $(".songTotalPlaytime").html(hrs+" hr "+mins+" mins");
            if (playlistSortDir == 1) {
                $(".titleRow .sortDir").remove();
                $(".titleRow ."+playlistSort).append("<span class='sortDir' data-sortdir='-1' data-sort='"+playlistSort+"'><i class=\"fa fa-chevron-down\" aria-hidden=\"true\"></i></div>");
                $("#playlist > li:not(.titleRow)").sort(function asc_sort(a, b) {
                    return ($(b).find("."+playlistSort).attr("data-sortIndex")) < ($(a).find("."+playlistSort).attr("data-sortIndex")) ? 1 : -1;
                }).appendTo('#playlist');
                $("#playlist > li:not(.titleRow)").each(function (index) {
                    track_no = index + 1
                    $(this).find(".trackRow .item").attr("data-trackno", track_no).html(track_no)
                });
            }
            else if (playlistSortDir == -1) {
                $(".titleRow .sortDir").remove();
                $(".titleRow ."+playlistSort).append("<span class='sortDir' data-sortdir='1' data-sort='"+playlistSort+"'><i class=\"fa fa-chevron-up\" aria-hidden=\"true\"></i></div>");
                $("#playlist > li:not(.titleRow)").sort(function dec_sort(a, b) {
                    return ($(b).find("."+playlistSort).attr("data-sortIndex")) > ($(a).find("."+playlistSort).attr("data-sortIndex")) ? 1 : -1;
                }).appendTo('#playlist');
                $("#playlist > li:not(.titleRow)").each(function(index){
                    track_no = index + 1
                    $(this).find(".trackRow .item").attr("data-trackno", track_no).html(track_no)
                });
            }
            $("#playlistMore").click(function(e){
                CurX = (window.Event) ? e.pageX : e.clientX + (document.documentElement.scrollLeft ? document.documentElement.scrollLeft : document.body.scrollLeft);
                CurY = (window.Event) ? e.pageY : e.clientY + (document.documentElement.scrollTop ? document.documentElement.scrollTop : document.body.scrollTop);
                $(".contextMenu").html("<ul></ul>");
                $(".contextMenu").css("display", "block").css("top", CurY + "px").css("left", CurX + "px");
                if("#{user.id}" == d.creator.toString()){
                    $(".contextMenu ul").append("<li>Copy Playlist Link</li><li>Copy Playlist ID</li><li class='sep'></li><li>Collaborative Playlist</li><li>Make Private</li><li class='sep'></li><li>Edit Details</li><li style='color: rgba(242, 52, 51, 1.00);' onclick=\"deletePlaylist('"+playlistID+"')\">Delete</li>");
                }
                else{
                    $(".contextMenu ul").append("<li>Copy Playlist Link</li><li>Copy Playlist ID</li><li class='sep'></li><li disabled='true'>Collaborative Playlist</li><li>Unfollow/Follow</li>");
                }
            });
            updateDblClicks();
            loadPlaying();
            loadRandom();
        },
        error: function(err){
            console.log(err);
        }
    });
}

function deletePlaylist(playlistID){
    $.ajax({
        method: "DELETE",
        url: '/api/playlist/' + playlistID + '?api_key=caf07b8b-366e-44ab-9bda-623f94a9c2df',
        dataType: 'json',
        beforeSend: function (xhr) {
            xhr.setRequestHeader("Authorization", "Bearer #{user.motorbotAccessToken}");
        },
        success: function (d) {
            loadView("playlists","undefined");
        },
        error: function(err){
            console.log(err)
        }
    });
}

function loadRandom(){
    $.ajax({
        url: '/api/getRandomPlayback',
        dataType: 'json',
        success: function (d) {
            if (d.randomPlayback) {
                $(".random").addClass("active");
            }
        }
    });
}

function loadPlaying(){
    $.ajax({
        url: '/api/music/playing?api_key=caf07b8b-366e-44ab-9bda-623f94a9c2df',
        dataType: 'json',
        success: function(data){
            if(data._id){
                if (data.title.length > 32) {
                    $(".activeTrack").html("<marquee>" + data.title + "</marquee>");
                }
                else {
                    $(".activeTrack").html(data.title);
                }
                $(".activeArtist").html(data.artist.name || "");
                if (data.artwork) {
                    $(".activeArtwork").attr("style", "background: url('" + data.artwork + "') no-repeat center; background-size: cover;")
                }
                else {
                    $(".activeArtwork").attr("style", "");
                }
                songDuration = data.duration
                if ($("#playlist").length) {
                    if ($("#playlist").attr("data-playlistId") == data.playlistId) {
                        $(".trackPlaying").removeClass("trackPlaying");
                        $(".play").parent().html($(".play").parent().attr('data-trackNo'));
                        $("#" + data.songId + " .item").html($("#" + data.songId + " .item").html()+'<i class="fa fa-play play" aria-hidden="true"></i>');
                        $("#" + data.songId).addClass("trackPlaying");
                        $(".playPlaylistButton").html('<i class="fa fa-pause" aria-hidden="true"></i> &nbsp; &nbsp;PAUSE').click(function (event) {
                            pauseMusic()
                        });
                    }
                }
                $("#playStop").html('<i class="fa fa-pause" aria-hidden="true" onclick="pauseMusic();" style="cursor: pointer;""></i>')
                $(".timeline").css("width","0%");
            }
        },
        error: function(err){
            console.log(err);
        }
    });
}

function playMusic(){
    $.ajax({
        url: '/api/music/play?api_key=caf07b8b-366e-44ab-9bda-623f94a9c2df',
        dataType: 'json',
        success: function(d){
            //setTimeout(function(){window.location.reload()},2000);
        },
        error: function(err){
            console.log(err);
        }
    });
}

function stopMusic(){
    $.ajax({
        url: '/api/music/stop?api_key=caf07b8b-366e-44ab-9bda-623f94a9c2df',
        dataType: 'json',
        success: function(d){
            //setTimeout(function(){window.location.reload()},2000);
        },
        error: function(err){
            console.log(err);
        }
    });
}

function prevMusic(){
    $.ajax({
        url: '/api/music/prev?api_key=caf07b8b-366e-44ab-9bda-623f94a9c2df',
        dataType: 'json',
        success: function(d){
            //setTimeout(function(){window.location.reload()},2000);
        },
        error: function(err){
            console.log(err);
        }
    });
}

function skipMusic(){
    $.ajax({
        url: '/api/music/skip?api_key=caf07b8b-366e-44ab-9bda-623f94a9c2df',
        dataType: 'json',
        success: function(d){
            //setTimeout(function(){window.location.reload()},2000);
        },
        error: function(err){
            console.log(err);
        }
    });
}

function pauseMusic() {
    $.ajax({
        url: '/api/music/pause?api_key=caf07b8b-366e-44ab-9bda-623f94a9c2df',
        dataType: 'json',
        success: function (d) {
            musicPaused = true;
            //setTimeout(function(){window.location.reload()},2000);
        },
        error: function (err) {
            console.log(err);
        }
    });
}

function resumeMusic() {
    $.ajax({
        url: '/api/music/play?api_key=caf07b8b-366e-44ab-9bda-623f94a9c2df',
        dataType: 'json',
        success: function (d) {
            //setTimeout(function(){window.location.reload()},2000);
        },
        error: function (err) {
            console.log(err);
        }
    });
}

function randomMusic(){
    $.ajax({
        url: '/api/toggleRandomPlayback',
        dataType: 'json',
        success: function (d) {

        }
    });
}

function repeatMusic() {
    /*if ($(".repeat").hasClass("active")) {
     $(".repeat").toggleClass("active");
     }
     else {
     $(".repeat").toggleClass("active");
     }*/
}