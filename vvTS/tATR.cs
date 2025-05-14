using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000009 RID: 9
	[HandlerCategory("vvIndicators"), HandlerName("Tick ATR"), InputInfo(0, "Цена")]
	public class tATR : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000043 RID: 67 RVA: 0x00002FAC File Offset: 0x000011AC
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("TickATR", new string[]
			{
				this.Period.ToString(),
				src.GetHashCode().ToString()
			}, () => tATR.GenTATR(src, this.Period));
		}

		// Token: 0x06000042 RID: 66 RVA: 0x00002F30 File Offset: 0x00001130
		public static IList<double> GenTATR(IList<double> src, int period)
		{
			double[] array = new double[src.Count];
			for (int i = 0; i < src.Count; i++)
			{
				if (i > 0)
				{
					array[i] = Math.Abs(src[i] - src[i - 1]);
				}
				else
				{
					array[i] = 0.0;
				}
			}
			return Series.EMA(array, period);
		}

		// Token: 0x17000014 RID: 20
		public IContext Context
		{
			// Token: 0x06000044 RID: 68 RVA: 0x00003018 File Offset: 0x00001218
			get;
			// Token: 0x06000045 RID: 69 RVA: 0x00003020 File Offset: 0x00001220
			set;
		}

		// Token: 0x17000013 RID: 19
		[HandlerParameter(true, "14", Min = "5", Max = "100", Step = "5")]
		public int Period
		{
			// Token: 0x06000040 RID: 64 RVA: 0x00002F1E File Offset: 0x0000111E
			get;
			// Token: 0x06000041 RID: 65 RVA: 0x00002F26 File Offset: 0x00001126
			set;
		}
	}
}
