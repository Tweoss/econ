supply_shock = input('supply_shock = ');
subsidies = input('subsidies = ');
trending = input('trending = ');
y_p = supply_shock-subsidies;
y_c = trending;

c_x = [0,40,65,80];
c_y = [80,80,70,0];

p_x = [0,10,45,80];
p_y = [80,-10,-10,100];

% c_x = [0,40,70,80];
% c_y = [80,80,70,0];

% p_x = [0,10,50,80];
% p_y = [80,-10,-10,100];

function value = calc_pos(t,a,b,c,d)
    value = (1-t)^3 * a  + 3*(1-t)^2*t * b + 3*(1-t)*t^2 * c + t^3 * d;
endfunction

function output = integrate(t,a,b,c,d,e,f,g,h,p)
    output = -3*(a - b)*(e + p)*t + (3/2)*(5*a*e - 7*b*e + 2*c*e - 3*a*f + 3*b*f + 2*(a - 2*b + c)*p)*t^2 - (9*c*e - d*e - 6*c*f + 3*c*p - d*p - 3*b*(6*e - 6*f + g + p) + a*(10*e - 12*f + 3*g + p))*t^3 + (3/4)*(3*(5*c*e - d*e - 7*c*f + d*f + 2*c*g) + a*(10*e - 18*f + 9*g - h) + b*(-22*e + 36*f - 15*g + h))*t^4 - (3/5)*(5*a*e - 13*b*e + 11*c*e - 3*d*e - 12*a*f + 30*b*f - 24*c*f + 6*d*f + 9*a*g - 21*b*g + 15*c*g - 3*d*g - 2*(a - 2*b + c)*h)*t^5 + (1/2)*(a - 3*b + 3*c - d)*(e - 3*f + 3*g - h)*t^6;
endfunction

function output = producer_surplus(t,a,b,c,d,e,f,g,h,p,x,y)
    output = x*y-integrate(t,a,b,c,d,e,f,g,h,p);
endfunction

function output = consumer_surplus(t,a,b,c,d,e,f,g,h,p,x,y)
    output = integrate(t,a,b,c,d,e,f,g,h,p) - x*y;
endfunction

function [err] = calc(t, y_p, y_c, p_x, p_y, c_x, c_y)
    xcoord_p = calc_pos(t(1),p_x(1),p_x(2),p_x(3),p_x(4));
    ycoord_p = calc_pos(t(1),p_y(1),p_y(2),p_y(3),p_y(4)) + y_p;
    xcoord_c = calc_pos(t(2),c_x(1),c_x(2),c_x(3),c_x(4));
    ycoord_c = calc_pos(t(2),c_y(1),c_y(2),c_y(3),c_y(4)) + y_c;
    err = (xcoord_p-xcoord_c)^2+(ycoord_p-ycoord_c)^2;
    % prevent the algorithm from staying at t = 0
    if xcoord_p < 0.5
	    err = 10000-xcoord_p;
    endif
endfunction
f = @(t) calc(t,y_p,y_c, p_x, p_y, c_x, c_y); 
[t, ferr] = fsolve(f, [0,0]);

x = calc_pos(t(1),p_x(1),p_x(2),p_x(3),p_x(4));
y = calc_pos(t(1),p_y(1),p_y(2),p_y(3),p_y(4));

disp(['t_p = ', num2str(t(1))])
disp(['t_c = ', num2str(t(2))])
disp(['x = ', num2str(calc_pos(t(1),p_x(1),p_x(2),p_x(3),p_x(4)))])
disp(['y = ', num2str(calc_pos(t(1),p_y(1),p_y(2),p_y(3),p_y(4)+y_p))])
% disp(['y = ', num2str((1-t(1))^3*(80+y_p) + 3*(1-t(1))^2*t(1)*(-10+y_p) + 3*(1-t(1))*t(1)^2*(-10+y_p) + t(1)^3*(100+y_p))])
disp(['err = ', num2str(ferr)])
disp(['equilibrium producer_surplus = ', num2str(producer_surplus(t(1), p_x(1),p_x(2),p_x(3),p_x(4),p_y(1),p_y(2),p_y(3),p_y(4),y_p,x,y))])
disp(['equilibrium consumer_surplus = ', num2str(consumer_surplus(t(2), c_x(1),c_x(2),c_x(3),c_x(4),c_y(1),c_y(2),c_y(3),c_y(4),y_c,x,y))])
disp(['necessary funds. consumer: ', num2str(x*y), ', producer: ', num2str(integrate(t(1),p_x(1),p_x(2),p_x(3),p_x(4),p_y(1),p_y(2),p_y(3),p_y(4),y_p))])

