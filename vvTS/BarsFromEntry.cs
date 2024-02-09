using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000EF RID: 239
	[HandlerCategory("vvTrade"), HandlerName("Баров со входа в тек. позицию")]
	public class BarsFromEntry : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x06000740 RID: 1856 RVA: 0x0002065F File Offset: 0x0001E85F
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			return (double)(barNum - pos.get_EntryBarNum());
		}
	}
}
