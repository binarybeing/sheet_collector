pub mod content{
    pub static REFRESH_BUTTON: &'static str = "<div></div><div id=\"x-spreadsheet-demo\"></div>";
    pub static ORTER: &'static str = "";
    pub static TABLE: &'static str = "
    
    <script type='text/javascript'>
    
    Date.prototype.format = function(format)
{
 var o = {
 'M+' : this.getMonth()+1, //month
 'd+' : this.getDate(),    //day
 'h+' : this.getHours(),   //hour
 'm+' : this.getMinutes(), //minute
 's+' : this.getSeconds(), //second
 'q+' : Math.floor((this.getMonth()+3)/3),  //quarter
 'S' : this.getMilliseconds() //millisecond
 }
 if(/(y+)/.test(format)) format=format.replace(RegExp.$1,
 (this.getFullYear()+'').substr(4 - RegExp.$1.length));
 for(var k in o)if(new RegExp(\"(\"+ k +\")\").test(format))
 format = format.replace(RegExp.$1,
 RegExp.$1.length==1 ? o[k] :
 ('00'+ o[k]).substr((''+ o[k]).length));
 return format;
}
    
    var task_name;
    var title_name;
    var xs;
    var load_time;
    window.onload = function(){
      load_time = new Date().getTime();
      task_name =  $('#the_task_name').val();
      title_name = $('#the_task_title').val();
      load();
    }
    
    
  
    function load(){
      // var rows_data = resultData.rows;
      // var order_config = eval(resultData.config.order_config);
      // var row_to_fill = {};
      // row_to_fill.len = 200;
      // row_to_fill[0] = {cells:buildHead(order_config)};
  
      // var index = 1;
      // for(var row of rows_data){
      //   var temp_row = {};
      //   var cells = {};
      //   var count = 0;
      //   for(var filed of order_config){
      //       // if(!row[filed+'']){
      //       //     continue;
      //       // }
      //       if('time_millisecond' == filed){
      //           var time_str= new Date(parseInt(row[''+filed]));
      //           cells[''+count] = {text:time_str.format('yyyy-MM-dd hh:mm:ss')};
      //       }else{
      //           cells[''+count] = {text:row[''+filed]};
      //       }
      //       count += 1;
      //   }
        
      //   temp_row.cells=cells;
      //   row_to_fill[''+index]=temp_row;
      //   index = index+1;
      // }
  
   
       //const rows =  row_to_fill;
       const rows =  {};
  
      const rows10 = { len: 1000 };
        for (let i = 0; i < 1000; i += 1) {
          rows10[i] = {
            cells: {
              0: { text: 'A-' + i },
              1: { text: 'B-' + i },
              2: { text: 'C-' + i },
              3: { text: 'D-' + i },
              4: { text: 'E-' + i },
              5: { text: 'F-' + i },
            }
          };
        }
        // x_spreadsheet.locale('zh-cn');
       xs = x_spreadsheet('#x-spreadsheet-demo', {showToolbar: true, showGrid: true})
          .loadData([{
            row: {
                height: 100,
                len: 100
              },
            styles: [
              {
                bgcolor: '#f4f5f8',
                textwrap: true,
                color: '#900b09',
                border: {
                  top: ['thin', '#0366d6'],
                  bottom: ['thin', '#0366d6'],
                  right: ['thin', '#0366d6'],
                  left: ['thin', '#0366d6'],
                },
              },
            ],
            rows,
          }]).change((cdata) => {
            console.log('>>>', xs.getData());
          });
  
        xs.on('cell-selected', (cell, ri, ci) => {
            console.log('cell:', cell, ', ri:', ri, ', ci:', ci);
          }).on('cell-edited', (text, ri, ci) => {
            console.log('text:', text, ', ri: ', ri, ', ci:', ci);
          });
        xs.change(function(data){
            console.log('data change',data);
        })

        update(xs);
    }
    function update(xs){
      const target_data = xs.datas.find(it=> it.name === 'sheet2');
      do_update(target_data);
      setInterval(
        function(){var target_data = xs.datas.find(it=> it.name === 'sheet2');do_update(target_data);},5000
      );
    }
    var data_cache=[];

    function do_update(target_data){
      if(!getLock()){
        return;
      }
      
      if(data_cache.length == 0){
        //初始尝试加载配置
        var core_data = fetchJson('/excel_core_data/'+ task_name+'/'+title_name );
        if(JSON.stringify(core_data) === '{}'){
          var data = fetchJson('/show_excel/'+ task_name+'/'+title_name); 
          var rows = target_data.rows;
          var row_data = rows.getData();
          row_data['0'] = {cells:buildHead(data.config.order_config)};
          rows.setData(row_data);
          xs.reRender();
          data_cache = data.rows;
          insert_rows(rows, data_cache,data.config.order_config);
          
        }else{
          console.log(core_data.core);
          xs.loadData(core_data.core);
          xs.reRender();
          data_cache = core_data.data_cache;
        }
        
      }else{
        //判断是否有新的数据
        var data = fetchJson('/show_excel/'+ task_name+'/'+title_name); 
        if(JSON.stringify(data)!='{}' && data_cache.length != data.rows.length){
          //insert_rows();
          var sheet_data = xs.datas.find(it=> it.name === 'sheet2');
          var rows = sheet_data.rows;
          try_insert_rows(rows,data_cache,data.rows,data.config.order_config);
          data_cache= data.rows;
        }
        const target_data = xs.datas.find(it=> it.name === 'sheet2');
        var len = Object.keys(target_data.rows).length;
        if(len != data_cache.length){

        }
        
      }
      pushJson('/sync_excel_core_data/'+task_name, {core:xs.getData(), data_cache: data_cache});
    }
    function try_insert_rows(rows,old_rows, new_rows, order_config){
      var arr = [];
      var maxTime = 0;
      for(var r of old_rows){
        if(r.time_millisecond > maxTime){
          maxTime = r.time_millisecond;
        }
      }
      console.log('maxTime',maxTime);
      for(var r of new_rows){
        if(r.time_millisecond > maxTime ){
          arr.push(r);
        }
      }
      insert_rows(rows, arr,order_config );

    }
    function insert_rows(rows, arr, order_config){
      
      // const target_data = xs.datas.find(it=> it.name === 'sheet2');
      // var maxIndex = 0;
      // for(var r of rows){
      //   console.log(r);
      // }
      var data = xs.getData();
      var sheet_rows = data[0].rows;
      console.log(sheet_rows);
      var keys = Object.keys(rows['_']);
      console.log(keys);
      var len = -1;
      if(keys.length>0){
        len = keys[keys.length-1];
        console.log(len);
      }
      len = parseInt(len)+1;

      
      var order = eval(order_config);
      
      var data = rows.getData();

      for(var row of arr){
        var temp_row = {};
        var cells = {};
        var count = 0;
        for(var filed of order){
            if('time_millisecond' == filed){
                var time_str= new Date(parseInt(row[''+filed]));
                cells[''+count] = {text:time_str.format('yyyy-MM-dd hh:mm:ss')};
            }else{
                cells[''+count] = {text:row[''+filed]};
            }
            count += 1;
        }
        
        temp_row.cells=cells;
        data[''+len] = temp_row;
        len = len+1;
      }
      rows.setData(data);
      xs.reRender();
      

    }
    function buildHead(order_config){
      var index = 0;
      var res = {};
      var obj = JSON.parse( order_config );
      for(var title of obj){
        res[''+index] = {text: title};
        index = index +1;
      }
      return res;
    }

    function pushJson(url, json){
      $.ajax({
        type: 'post',
        url: url,
        contentType: 'application/json; charset=utf-8',
        cache: false,
        withCredentials: true,
        async: false,
        data: JSON.stringify(json),
        success: function(resultData) {
            
        },
        error: function(result) {
            
        }
      })
    }
    function fetchJson(url){
      var result = {};
      $.ajax({
        type: 'get',
        url: url,
        withCredentials: true,
        async: false,
        success: function(resultData) {
              result = resultData;
        },
        error: function(e){
          result = {};
        }
      });
      
      return result;
    }
    function getLock(){
      var res = false;
      $.ajax({
        type: 'get',
        url: '/excel_core_data_lock/'+task_name+'/'+load_time,
        withCredentials: true,
        async: false,
        success: function(resultData) {
              //console.log('get lock success',resultData);
              res = true;
        },
        error: function(e){
            //console.log('get lock error',res);
            alert('已由其他页面加载或服务器关闭，请刷新重试');
        }
      });
      console.log('get lock ',res);
      return res;
    }
  </script>

   ";
}