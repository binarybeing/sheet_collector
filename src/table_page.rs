pub mod content{
    pub static REFRESH_BUTTON: &'static str = "<div></div><div id=\"x-spreadsheet-demo\"></div>";
    pub static ORTER: &'static str = "";
    pub static TABLE: &'static str = "
    
    <script type='text/javascript'>
    function refresh(data){
  
    }
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
  
    window.onload = function(){
      var task_name = $('#the_task_name').val();
      var task_title = $('#the_task_title').val();
      var url = '/show_excel/'+task_name+'/'+task_title;
      $.ajax({
        type: 'get',
        url: url,
        withCredentials: true,
        async: false,
        success: function(resultData) {
              //var config = getConfig(task_name);
              load(resultData);
        }
      })
    }
    function buildHead(order_config){
      var index = 0;
      var res = {};
      for(var title of order_config){
        res[''+index] = {text: title};
        index = index +1;
      }
      return res;
    }
  
    function load(resultData){
      var rows_data = resultData.rows;
      var order_config = eval(resultData.config.order_config);
      var row_to_fill = {};
      row_to_fill.len = 200;
      row_to_fill[0] = {cells:buildHead(order_config)};
  
      var index = 1;
      for(var row of rows_data){
        var temp_row = {};
        var cells = {};
        var count = 0;
        for(var filed of order_config){
            // if(!row[filed+'']){
            //     continue;
            // }
            if('time_millisecond' == filed){
                var time_str= new Date(parseInt(row[''+filed]));
                cells[''+count] = {text:time_str.format('yyyy-MM-dd hh:mm:ss')};
            }else{
                cells[''+count] = {text:row[''+filed]};
            }
            count += 1;
        }
        
        temp_row.cells=cells;
        row_to_fill[''+index]=temp_row;
        index = index+1;
      }
  
   
       const rows =  row_to_fill;
  
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
        var xs = x_spreadsheet('#x-spreadsheet-demo', {showToolbar: true, showGrid: true})
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
  
    }
  </script>

   ";
}