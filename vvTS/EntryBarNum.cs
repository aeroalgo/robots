using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000FA RID: 250
	[HandlerCategory("vvTrade"), HandlerName("Номер бара входа в позицию")]
	public class EntryBarNum : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x0600075A RID: 1882 RVA: 0x000208FD File Offset: 0x0001EAFD
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			return (double)pos.get_EntryBarNum();
		}
	}
}
