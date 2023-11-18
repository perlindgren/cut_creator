clear

N=200;	%number of input samples

f=0.5; %input test signal frequency,  [0..1] ( *fs/2), 


tsamp2=0.5;	%output sample time interval (fraction of input sample duration=1)


w=f *pi;
t=0:N;	%input sample time vector 
u=sin(w*t).*hanning(length(t))';  %input signal vector, windowed
%u=sin(w*t);  %input signal vector, plain


t2=0:tsamp2:N;



dt=20;  %sinc and window length in input sample time units



for k2=1:length(t2)-1

  [a k]=min(abs(t-t2(k2))); %closest input sample

  if k==length(t)
    continue
  end

  ts=t(k+1)-t(k);  %local input sample rate

  ts2=t2(k2+1)-t2(k2); %local output sample rate

  %sinc stretch factor
  if ts2<ts
    s=1;  %increasing sample rate
  else
    s=ts2/ts; %decreasing, need larger window
  end


  [a x1]=min(abs(t-t(k)+dt*s)); %first and last sample in window 
  [a x2]=min(abs(t-t(k)-dt*s)); 


  d=0; %accumulates sum of weighted samples
  n=0; %accumulates sum of weights for normalization

  for k1=x1:x2
    ts=0.9*(t(k1)-t2(k2) )/s;   %time of each input sample relative to output sample time, stretched to make filter roll off slighty earlier to suppress folding from tones close to fs/2
 
    w1=sinc(ts);		%low-pass filter 
    w2=(1+cos( pi* ts/dt) );	%window function (hann)

    d=d+u(k1)*w1*w2;
    n=n+w1*w2;
  end

  y(k2)=d/n; 	%finished output sample
end


figure(1)
clf
plot(t2(1:length(y)),y,'-or')
hold on
plot(t,u,'--*b')
grid on
legend('output','input');
xlabel('time (input samples)');


figure(2)
clf


U=20*log10(abs(fft([u.*hanning(length(u))' zeros(1,length(u)*3)]))/length(t) )+12;
plot( ((1:length(U)/2)-1)/length(U),U(1:end/2));

hold on
Y=20*log10(abs(fft([y.*hanning(length(y))' zeros(1,length(u)*3)] ))/length(t2) ) +12;
plot( ((1:floor(length(Y)/2))-1)/length(Y)/tsamp2,Y(1:floor(end/2)));
grid on

axis([0 0.5*max(1,length(y)/length(u)) -103 3])
legend('input','output');
xlabel('frequency (1/fs)')









