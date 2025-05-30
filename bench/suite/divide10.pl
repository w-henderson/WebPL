% generated: 7 March 1990
% option(s):
%
%   (deriv) divide10
%
%   David H. D. Warren
%   Copyright: Public domain
%
%   symbolic derivative of ((((((((x/x)/x)/x)/x)/x)/x)/x)/x)/x

top:-divide10.


divide10 :- d(((((((((x/x)/x)/x)/x)/x)/x)/x)/x)/x,x,_).

d(U+V,X,DU+DV) :- !,
    d(U,X,DU),
    d(V,X,DV).
d(U-V,X,DU-DV) :- !,
    d(U,X,DU),
    d(V,X,DV).
d(U*V,X,DU*V+U*DV) :- !,
    d(U,X,DU),
    d(V,X,DV).
d(U/V,X,(DU*V-U*DV)/(pow(V,2))) :- !,
    d(U,X,DU),
    d(V,X,DV).
d(pow(U,N),X,DU*N*(pow(U,N1))) :- !,
    integer(N),
    N1 is N-1,
    d(U,X,DU).
d(-U,X,-DU) :- !,
    d(U,X,DU).
d(exp(U),X,exp(U)*DU) :- !,
    d(U,X,DU).
d(log(U),X,DU/U) :- !,
    d(U,X,DU).
d(X,X,1) :- !.
d(_,_,0).
