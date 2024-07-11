export const idlFactory = ({ IDL }) => {
  return IDL.Service({ 'sign' : IDL.Func([IDL.Text], [IDL.Text], []) });
};
export const init = ({ IDL }) => { return []; };
