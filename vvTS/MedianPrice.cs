using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000035 RID: 53
	[HandlerCategory("vvIndicators"), HandlerName("Median price")]
	public class MedianPrice : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060001EC RID: 492 RVA: 0x00009348 File Offset: 0x00007548
		public IList<double> Execute(ISecurity src)
		{
			return this.Context.GetData("MedianPrice", new string[]
			{
				src.get_CacheName()
			}, () => Series.MedianPrice(src.get_Bars()));
		}

		// Token: 0x060001ED RID: 493 RVA: 0x00009398 File Offset: 0x00007598
		public static IList<double> GenMedianPrice(IList<Bar> candles)
		{
			int count = candles.Count;
			double[] array = new double[count];
			for (int i = 0; i < count; i++)
			{
				array[i] = (candles[i].get_High() + candles[i].get_Low()) / 2.0;
			}
			return array;
		}

		// Token: 0x170000A7 RID: 167
		public IContext Context
		{
			// Token: 0x060001EE RID: 494 RVA: 0x000093E6 File Offset: 0x000075E6
			get;
			// Token: 0x060001EF RID: 495 RVA: 0x000093EE File Offset: 0x000075EE
			set;
		}
	}
}
