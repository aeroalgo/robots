using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000067 RID: 103
	[HandlerCategory("vvIndicators"), HandlerName("Typical Price")]
	public class TypicalPrice : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs
	{
		// Token: 0x0600039B RID: 923 RVA: 0x00014443 File Offset: 0x00012643
		public IList<double> Execute(ISecurity src)
		{
			return TypicalPrice.GenTypicalPrice(src.get_Bars());
		}

		// Token: 0x0600039A RID: 922 RVA: 0x000143E8 File Offset: 0x000125E8
		public static IList<double> GenTypicalPrice(IList<Bar> candles)
		{
			int count = candles.Count;
			double[] array = new double[count];
			for (int i = 0; i < count; i++)
			{
				array[i] = (candles[i].get_High() + candles[i].get_Low() + candles[i].get_Close()) / 3.0;
			}
			return array;
		}
	}
}
