using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000FC RID: 252
	[HandlerCategory("vvTrade"), HandlerName("Номер бара выхода из позиции")]
	public class ExitBarNum : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x0600075E RID: 1886 RVA: 0x00020952 File Offset: 0x0001EB52
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			return (double)pos.get_ExitBarNum();
		}
	}
}
