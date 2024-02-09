using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200014B RID: 331
	[HandlerCategory("vvMACD"), HandlerName("MACD(DEMA)DiNapoli")]
	public class MACDDemaDiNapoli : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x06000A37 RID: 2615 RVA: 0x0002A8DC File Offset: 0x00028ADC
		public IList<double> Execute(IList<double> src)
		{
			int count = src.Count;
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			double[] array4 = new double[count];
			double[] array5 = new double[count];
			for (int i = 1; i < count; i++)
			{
				array3[i] = array3[i - 1] + 2.0 / (1.0 + this.FastEMA) * (src[i] - array3[i - 1]);
				array4[i] = array4[i - 1] + 2.0 / (1.0 + this.SlowEMA) * (src[i] - array4[i - 1]);
				array[i] = array3[i] - array4[i];
				array2[i] = array2[i - 1] + 2.0 / (1.0 + this.SignalEMA) * (array[i] - array2[i - 1]);
				array5[i] = 0.0;
				if (array[i] > array2[i])
				{
					array5[i] = 1.0;
				}
				if (array[i] < array2[i])
				{
					array5[i] = -1.0;
				}
			}
			if (this.Output == 1)
			{
				return array2;
			}
			if (this.Output == 2)
			{
				return array3;
			}
			if (this.Output == 3)
			{
				return array4;
			}
			if (this.Output == 4)
			{
				return array5;
			}
			return array;
		}

		// Token: 0x17000357 RID: 855
		[HandlerParameter(true, "8.3896", Min = "3", Max = "20", Step = "1")]
		public double FastEMA
		{
			// Token: 0x06000A2F RID: 2607 RVA: 0x0002A896 File Offset: 0x00028A96
			get;
			// Token: 0x06000A30 RID: 2608 RVA: 0x0002A89E File Offset: 0x00028A9E
			set;
		}

		// Token: 0x1700035A RID: 858
		[HandlerParameter(true, "0", Min = "0", Max = "3", Step = "1")]
		public int Output
		{
			// Token: 0x06000A35 RID: 2613 RVA: 0x0002A8C9 File Offset: 0x00028AC9
			get;
			// Token: 0x06000A36 RID: 2614 RVA: 0x0002A8D1 File Offset: 0x00028AD1
			set;
		}

		// Token: 0x17000359 RID: 857
		[HandlerParameter(true, "9.0503", Min = "3", Max = "20", Step = "1")]
		public double SignalEMA
		{
			// Token: 0x06000A33 RID: 2611 RVA: 0x0002A8B8 File Offset: 0x00028AB8
			get;
			// Token: 0x06000A34 RID: 2612 RVA: 0x0002A8C0 File Offset: 0x00028AC0
			set;
		}

		// Token: 0x17000358 RID: 856
		[HandlerParameter(true, "17.5185", Min = "3", Max = "20", Step = "1")]
		public double SlowEMA
		{
			// Token: 0x06000A31 RID: 2609 RVA: 0x0002A8A7 File Offset: 0x00028AA7
			get;
			// Token: 0x06000A32 RID: 2610 RVA: 0x0002A8AF File Offset: 0x00028AAF
			set;
		}
	}
}
