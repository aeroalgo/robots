using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200013C RID: 316
	[HandlerCategory("vvRSI"), HandlerName("Synthetic RSI")]
	public class SyntRSI2 : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x0600099F RID: 2463 RVA: 0x000281BC File Offset: 0x000263BC
		public IList<double> Execute(IList<double> src)
		{
			return SyntRSI2.GenSyntRSI(src, this.emaLength1, this.rsiLength1, this.emaLength2, this.rsiLength2, this.emaLength3, this.rsiLength3, this.rsiSignalLength, this.maMethod, this.Context, this.Output, this.postSmooth);
		}

		// Token: 0x0600099E RID: 2462 RVA: 0x00028018 File Offset: 0x00026218
		public static IList<double> GenSyntRSI(IList<double> src, int _emaLength1, int _rsiLength1, int _emaLength2, int _rsiLength2, int _emaLength3, int _rsiLength3, int _rsiSignalLength, int _maMethod, IContext context, int _Output, int _postSmooth)
		{
			int count = src.Count;
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			double[] array4 = new double[count];
			double[] array5 = new double[count];
			int num = Math.Max(Math.Max(_emaLength1, _emaLength2), _emaLength3);
			for (int i = num; i < count; i++)
			{
				array3[i] = vvSeries.iMA(src, array3, _maMethod, _emaLength1, i, 1.0, 0.0);
				array4[i] = vvSeries.iMA(src, array4, _maMethod, _emaLength2, i, 1.0, 0.0);
				array5[i] = vvSeries.iMA(src, array5, _maMethod, _emaLength3, i, 1.0, 0.0);
			}
			num = Math.Max(Math.Max(_rsiLength1, _rsiLength2), _rsiLength3);
			IList<double> list = RSI.GenRSI(array3, _rsiLength1, 0, 0, 100, false);
			IList<double> list2 = RSI.GenRSI(array4, _rsiLength2, 0, 0, 100, false);
			IList<double> list3 = RSI.GenRSI(array5, _rsiLength3, 0, 0, 100, false);
			for (int j = 0; j < count; j++)
			{
				array[j] = (list3[j] + 2.0 * list2[j] + 3.0 * list[j]) / 6.0;
			}
			for (int k = _rsiSignalLength; k < count; k++)
			{
				array2[k] = vvSeries.iMA(array, array2, _maMethod, _rsiSignalLength, k, 1.0, 0.0);
			}
			IList<double> result = array;
			if (_postSmooth > 0)
			{
				result = JMA.GenJMA(array, _postSmooth, 100);
			}
			if (_Output > 0)
			{
				return array2;
			}
			return result;
		}

		// Token: 0x17000323 RID: 803
		public IContext Context
		{
			// Token: 0x060009A0 RID: 2464 RVA: 0x00028211 File Offset: 0x00026411
			get;
			// Token: 0x060009A1 RID: 2465 RVA: 0x00028219 File Offset: 0x00026419
			set;
		}

		// Token: 0x17000319 RID: 793
		[HandlerParameter(true, "48", Min = "3", Max = "50", Step = "1")]
		public int emaLength1
		{
			// Token: 0x0600098A RID: 2442 RVA: 0x00027F6E File Offset: 0x0002616E
			get;
			// Token: 0x0600098B RID: 2443 RVA: 0x00027F76 File Offset: 0x00026176
			set;
		}

		// Token: 0x1700031B RID: 795
		[HandlerParameter(true, "24", Min = "3", Max = "50", Step = "1")]
		public int emaLength2
		{
			// Token: 0x0600098E RID: 2446 RVA: 0x00027F90 File Offset: 0x00026190
			get;
			// Token: 0x0600098F RID: 2447 RVA: 0x00027F98 File Offset: 0x00026198
			set;
		}

		// Token: 0x1700031D RID: 797
		[HandlerParameter(true, "12", Min = "3", Max = "50", Step = "1")]
		public int emaLength3
		{
			// Token: 0x06000992 RID: 2450 RVA: 0x00027FB2 File Offset: 0x000261B2
			get;
			// Token: 0x06000993 RID: 2451 RVA: 0x00027FBA File Offset: 0x000261BA
			set;
		}

		// Token: 0x17000320 RID: 800
		[HandlerParameter(true, "1", Min = "0", Max = "5", Step = "1")]
		public int maMethod
		{
			// Token: 0x06000998 RID: 2456 RVA: 0x00027FE5 File Offset: 0x000261E5
			get;
			// Token: 0x06000999 RID: 2457 RVA: 0x00027FED File Offset: 0x000261ED
			set;
		}

		// Token: 0x17000321 RID: 801
		[HandlerParameter(true, "0", Min = "0", Max = "1", Step = "1")]
		public int Output
		{
			// Token: 0x0600099A RID: 2458 RVA: 0x00027FF6 File Offset: 0x000261F6
			get;
			// Token: 0x0600099B RID: 2459 RVA: 0x00027FFE File Offset: 0x000261FE
			set;
		}

		// Token: 0x17000322 RID: 802
		[HandlerParameter(true, "5", Min = "0", Max = "20", Step = "1")]
		public int postSmooth
		{
			// Token: 0x0600099C RID: 2460 RVA: 0x00028007 File Offset: 0x00026207
			get;
			// Token: 0x0600099D RID: 2461 RVA: 0x0002800F File Offset: 0x0002620F
			set;
		}

		// Token: 0x1700031A RID: 794
		[HandlerParameter(true, "32", Min = "3", Max = "50", Step = "1")]
		public int rsiLength1
		{
			// Token: 0x0600098C RID: 2444 RVA: 0x00027F7F File Offset: 0x0002617F
			get;
			// Token: 0x0600098D RID: 2445 RVA: 0x00027F87 File Offset: 0x00026187
			set;
		}

		// Token: 0x1700031C RID: 796
		[HandlerParameter(true, "16", Min = "3", Max = "50", Step = "1")]
		public int rsiLength2
		{
			// Token: 0x06000990 RID: 2448 RVA: 0x00027FA1 File Offset: 0x000261A1
			get;
			// Token: 0x06000991 RID: 2449 RVA: 0x00027FA9 File Offset: 0x000261A9
			set;
		}

		// Token: 0x1700031E RID: 798
		[HandlerParameter(true, "8", Min = "3", Max = "50", Step = "1")]
		public int rsiLength3
		{
			// Token: 0x06000994 RID: 2452 RVA: 0x00027FC3 File Offset: 0x000261C3
			get;
			// Token: 0x06000995 RID: 2453 RVA: 0x00027FCB File Offset: 0x000261CB
			set;
		}

		// Token: 0x1700031F RID: 799
		[HandlerParameter(true, "8", Min = "3", Max = "50", Step = "1")]
		public int rsiSignalLength
		{
			// Token: 0x06000996 RID: 2454 RVA: 0x00027FD4 File Offset: 0x000261D4
			get;
			// Token: 0x06000997 RID: 2455 RVA: 0x00027FDC File Offset: 0x000261DC
			set;
		}
	}
}
