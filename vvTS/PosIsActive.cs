using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000F5 RID: 245
	[HandlerCategory("vvTrade"), HandlerName("Эта позиция активна? (1/0)")]
	public class PosIsActive : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x06000750 RID: 1872 RVA: 0x00020821 File Offset: 0x0001EA21
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			return (double)(pos.IsActiveForbar(barNum) ? 1 : 0);
		}
	}
}
