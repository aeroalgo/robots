using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200011E RID: 286
	[HandlerCategory("vvBands&Channels"), HandlerDecimals(2), HandlerName("BBWidth")]
	public class BBWidth : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x06000828 RID: 2088 RVA: 0x00022D15 File Offset: 0x00020F15
		public IList<double> Execute(IList<double> src)
		{
			return BBWidth.GenBBWidth(src, this.BBPeriod, this.StdDeviation, this.NormPeriod);
		}

		// Token: 0x06000827 RID: 2087 RVA: 0x00022C24 File Offset: 0x00020E24
		public static IList<double> GenBBWidth(IList<double> src, int _BBPeriod, double _StdDeviation, int _NormPeriod = 0)
		{
			int count = src.Count;
			double[] array = new double[count];
			IList<double> result = array;
			for (int i = 0; i < count; i++)
			{
				array[i] = (BBands.iBBands(src, _BBPeriod, _StdDeviation, 1, i) - BBands.iBBands(src, _BBPeriod, _StdDeviation, 2, i)) / SMA.iSMA(src, _BBPeriod, i) * 100.0;
			}
			if (_NormPeriod > 0)
			{
				IList<double> list = array;
				double[] array2 = new double[count];
				result = array2;
				IList<double> list2 = vvSeries.Lowest(list, _NormPeriod);
				IList<double> list3 = vvSeries.Highest(list, _NormPeriod);
				for (int j = 0; j < count; j++)
				{
					double num = list2[j];
					double num2 = list3[j];
					if (num != num2)
					{
						array2[j] = 100.0 * ((list[j] - num) / (num2 - num));
					}
					else
					{
						array2[j] = 50.0;
					}
				}
			}
			return result;
		}

		// Token: 0x17000293 RID: 659
		[HandlerParameter(true, "20", Min = "10", Max = "40", Step = "1")]
		public int BBPeriod
		{
			// Token: 0x06000821 RID: 2081 RVA: 0x00022BF1 File Offset: 0x00020DF1
			get;
			// Token: 0x06000822 RID: 2082 RVA: 0x00022BF9 File Offset: 0x00020DF9
			set;
		}

		// Token: 0x17000295 RID: 661
		[HandlerParameter(true, "0", Min = "50", Max = "100", Step = "10")]
		public int NormPeriod
		{
			// Token: 0x06000825 RID: 2085 RVA: 0x00022C13 File Offset: 0x00020E13
			get;
			// Token: 0x06000826 RID: 2086 RVA: 0x00022C1B File Offset: 0x00020E1B
			set;
		}

		// Token: 0x17000294 RID: 660
		[HandlerParameter(true, "2", Min = "1", Max = "5", Step = "1")]
		public double StdDeviation
		{
			// Token: 0x06000823 RID: 2083 RVA: 0x00022C02 File Offset: 0x00020E02
			get;
			// Token: 0x06000824 RID: 2084 RVA: 0x00022C0A File Offset: 0x00020E0A
			set;
		}
	}
}
