cd /home/cc/TimeSeriesDB/database;
#dir=$1
#for comp in ofsgorilla gorilla gorillabd splitbd split bp zlib paa fourier snappy deflate gzip deltabp;
TIME=$4
SCL=$2
PRED=$3
for comp in gorilla gorillabd splitbd bp bpsplit gzip snappy sprintz plain dict pqgzip pqsnappy;
do
  for i in $(seq 1 $TIME);
		do
		  echo $i
#			for file in $(ls /mnt/hdd-2T-3/chunwei/timeseries_dataset/*/*/*);
      for file in $1;
			    do

            cargo +nightly run --release  --package time_series_start --bin comp_profiler $file $comp $SCL $PRED

			    done

		done

done
echo "Float compression done!"
python ./script/python/logparser.py nohup.out performance.csv $TIME
			